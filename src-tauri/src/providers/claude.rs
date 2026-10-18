//! Claude usage from the OAuth endpoint used by Claude Code.
//!
//! The access token is read from Claude Code's credentials file on every fetch, so a
//! token refreshed by Claude Code is picked up automatically. The file is never written.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::StatusCode;
use serde_json::Value;

use super::UsageProvider;
use crate::domain::{ProviderId, UsageWindow, WindowKind};
use crate::error::{ErrorCode, ProviderError};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA_HEADER: &str = "oauth-2025-04-20";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const USER_AGENT: &str = concat!("ai-usage/", env!("CARGO_PKG_VERSION"));

/// Response keys mapped to windows; any other key is ignored.
const WINDOW_KEYS: [(&str, WindowKind); 2] = [
    ("five_hour", WindowKind::Rolling5h),
    ("seven_day", WindowKind::Rolling7d),
];

const RENEW_HINT: &str = "Abra o Claude Code para renová-lo.";

pub struct ClaudeOAuthProvider {
    credentials_path: PathBuf,
    client: reqwest::Client,
}

impl ClaudeOAuthProvider {
    pub fn new(credentials_path: PathBuf) -> reqwest::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()?;
        Ok(Self {
            credentials_path,
            client,
        })
    }

    /// Location of Claude Code's credentials file inside the user's home directory.
    pub fn credentials_path(home: &Path) -> PathBuf {
        home.join(".claude").join(".credentials.json")
    }
}

#[async_trait]
impl UsageProvider for ClaudeOAuthProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    async fn fetch(&self) -> Result<Vec<UsageWindow>, ProviderError> {
        let credentials = read_credentials(&self.credentials_path)?;
        credentials.ensure_not_expired(Utc::now())?;

        let response = self
            .client
            .get(USAGE_URL)
            .bearer_auth(credentials.access_token.expose())
            .header("anthropic-beta", BETA_HEADER)
            .send()
            .await
            .map_err(network_error)?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(ProviderError::new(
                    ErrorCode::Unauthorized,
                    format!(
                        "Token do Claude recusado (HTTP {}). {RENEW_HINT}",
                        response.status().as_u16()
                    ),
                ));
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(ProviderError::new(
                    ErrorCode::RateLimited,
                    "Limite de consultas ao endpoint de uso atingido (HTTP 429). \
                     Nova tentativa no próximo ciclo.",
                ));
            }
            status if !status.is_success() => {
                return Err(ProviderError::new(
                    ErrorCode::Http,
                    format!(
                        "Resposta inesperada do servidor (HTTP {}).",
                        status.as_u16()
                    ),
                ));
            }
            _ => {}
        }

        let body = response.text().await.map_err(network_error)?;
        parse_usage(&body)
    }
}

/// Access token wrapper whose `Debug` output never reveals the value.
struct AccessToken(String);

impl AccessToken {
    fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccessToken(***)")
    }
}

#[derive(Debug)]
struct Credentials {
    access_token: AccessToken,
    expires_at: Option<DateTime<Utc>>,
}

impl Credentials {
    fn ensure_not_expired(&self, now: DateTime<Utc>) -> Result<(), ProviderError> {
        match self.expires_at {
            Some(expires_at) if expires_at <= now => Err(ProviderError::new(
                ErrorCode::TokenExpired,
                format!("Token do Claude expirado. {RENEW_HINT}"),
            )),
            _ => Ok(()),
        }
    }
}

fn read_credentials(path: &Path) -> Result<Credentials, ProviderError> {
    // The file is tiny; a blocking read does not stall the async runtime in practice.
    let raw = std::fs::read_to_string(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => ProviderError::new(
            ErrorCode::CredentialsMissing,
            format!(
                "Credenciais do Claude Code não encontradas em {}. Faça login no Claude Code.",
                path.display()
            ),
        ),
        kind => ProviderError::new(
            ErrorCode::CredentialsInvalid,
            format!(
                "Não foi possível ler as credenciais em {} ({kind}).",
                path.display()
            ),
        ),
    })?;
    parse_credentials(&raw)
}

/// Parses the credentials file. Error messages never include file contents,
/// since parser messages could echo a token value.
fn parse_credentials(raw: &str) -> Result<Credentials, ProviderError> {
    let invalid = || {
        ProviderError::new(
            ErrorCode::CredentialsInvalid,
            "Arquivo de credenciais do Claude Code em formato inesperado.",
        )
    };

    let root: Value = serde_json::from_str(raw).map_err(|_| invalid())?;
    let oauth = root.get("claudeAiOauth").ok_or_else(invalid)?;
    let access_token = oauth
        .get("accessToken")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())
        .ok_or_else(invalid)?;
    let expires_at = oauth
        .get("expiresAt")
        .and_then(Value::as_i64)
        .and_then(|millis| Utc.timestamp_millis_opt(millis).single());

    Ok(Credentials {
        access_token: AccessToken(access_token.to_owned()),
        expires_at,
    })
}

fn parse_usage(body: &str) -> Result<Vec<UsageWindow>, ProviderError> {
    let parse_error = |detail: &str| {
        ProviderError::new(
            ErrorCode::Parse,
            format!("Resposta do endpoint de uso em formato inesperado: {detail}."),
        )
    };

    let root: Value = serde_json::from_str(body).map_err(|_| parse_error("JSON inválido"))?;
    if !root.is_object() {
        return Err(parse_error("o corpo não é um objeto"));
    }

    let windows: Vec<UsageWindow> = WINDOW_KEYS
        .iter()
        .filter_map(|&(key, kind)| parse_window(root.get(key)?, kind))
        .collect();

    if windows.is_empty() {
        return Err(parse_error("nenhuma janela de uso reconhecida"));
    }
    Ok(windows)
}

/// Returns `None` when the window has no numeric utilization; a missing or
/// invalid `resets_at` keeps the window with an unknown reset time.
fn parse_window(value: &Value, kind: WindowKind) -> Option<UsageWindow> {
    let utilization = value.get("utilization")?.as_f64()?.clamp(0.0, 100.0);
    let resets_at = value
        .get("resets_at")
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|date| date.with_timezone(&Utc));

    Some(UsageWindow {
        provider: ProviderId::Claude,
        kind,
        utilization,
        resets_at,
    })
}

fn network_error(error: reqwest::Error) -> ProviderError {
    let message = if error.is_timeout() {
        "Tempo esgotado ao consultar o uso do Claude.".to_owned()
    } else {
        format!("Falha de rede ao consultar o uso do Claude: {error}")
    };
    ProviderError::new(ErrorCode::Network, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trimmed copy of a real response (2026-10-18), including unknown and null keys.
    const REAL_RESPONSE: &str = r#"{
        "five_hour": {"utilization": 0.0, "resets_at": "2026-10-19T01:00:00.422895+00:00",
                      "limit_dollars": null, "used_dollars": null, "locked_reason": null},
        "seven_day": {"utilization": 40.0, "resets_at": "2026-10-22T22:00:00.422915+00:00",
                      "limit_dollars": null, "used_dollars": null, "locked_reason": null},
        "seven_day_oauth_apps": null,
        "seven_day_opus": null,
        "seven_day_sonnet": null,
        "extra_usage": {"is_enabled": false, "monthly_limit": null},
        "limits": [{"kind": "session", "percent": 0}]
    }"#;

    fn utc(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn parses_real_response_ignoring_unknown_keys() {
        let windows = parse_usage(REAL_RESPONSE).unwrap();

        assert_eq!(
            windows,
            vec![
                UsageWindow {
                    provider: ProviderId::Claude,
                    kind: WindowKind::Rolling5h,
                    utilization: 0.0,
                    resets_at: Some(utc("2026-10-19T01:00:00.422895Z")),
                },
                UsageWindow {
                    provider: ProviderId::Claude,
                    kind: WindowKind::Rolling7d,
                    utilization: 40.0,
                    resets_at: Some(utc("2026-10-22T22:00:00.422915Z")),
                },
            ]
        );
    }

    #[test]
    fn null_or_missing_window_is_skipped() {
        let windows =
            parse_usage(r#"{"five_hour": null, "seven_day": {"utilization": 12.5}}"#).unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].kind, WindowKind::Rolling7d);
        assert_eq!(windows[0].utilization, 12.5);
    }

    #[test]
    fn windows_without_numeric_utilization_are_skipped() {
        let body = r#"{
            "five_hour": {"utilization": null, "resets_at": "2026-10-19T01:00:00+00:00"},
            "seven_day": {"utilization": "40", "resets_at": "2026-10-22T22:00:00+00:00"},
            "seven_day_sonnet": {"utilization": 99.0}
        }"#;

        assert_eq!(parse_usage(body).unwrap_err().code, ErrorCode::Parse);
    }

    #[test]
    fn invalid_or_null_reset_keeps_window() {
        let body = r#"{
            "five_hour": {"utilization": 5.0, "resets_at": null},
            "seven_day": {"utilization": 6.0, "resets_at": "next tuesday"}
        }"#;

        let windows = parse_usage(body).unwrap();

        assert_eq!(windows.len(), 2);
        assert!(windows.iter().all(|w| w.resets_at.is_none()));
    }

    #[test]
    fn utilization_is_clamped() {
        let body = r#"{"five_hour": {"utilization": 130.0}, "seven_day": {"utilization": -2}}"#;

        let windows = parse_usage(body).unwrap();

        assert_eq!(windows[0].utilization, 100.0);
        assert_eq!(windows[1].utilization, 0.0);
    }

    #[test]
    fn malformed_bodies_are_parse_errors() {
        for body in ["", "not json", "[]", "null", r#"{"unrelated": 1}"#] {
            assert_eq!(
                parse_usage(body).unwrap_err().code,
                ErrorCode::Parse,
                "{body}"
            );
        }
    }

    #[test]
    fn parses_credentials() {
        let raw = r#"{"claudeAiOauth": {"accessToken": "secret-token", "expiresAt": 1792380161082,
                      "refreshToken": "other", "scopes": []}, "organizationUuid": "x"}"#;

        let credentials = parse_credentials(raw).unwrap();

        assert_eq!(credentials.access_token.expose(), "secret-token");
        assert_eq!(
            credentials.expires_at,
            Some(utc("2026-10-19T03:22:41.082Z"))
        );
    }

    #[test]
    fn invalid_credentials_are_reported_without_contents() {
        let cases = [
            "not json",
            r#"{}"#,
            r#"{"claudeAiOauth": {}}"#,
            r#"{"claudeAiOauth": {"accessToken": ""}}"#,
            r#"{"claudeAiOauth": {"accessToken": 42}}"#,
        ];
        for raw in cases {
            let error = parse_credentials(raw).unwrap_err();
            assert_eq!(error.code, ErrorCode::CredentialsInvalid, "{raw}");
            assert!(!error.message.contains("42"));
        }
    }

    #[test]
    fn missing_credentials_file() {
        let path = std::env::temp_dir().join("ai-usage-missing-dir/.credentials.json");

        let error = read_credentials(&path).unwrap_err();

        assert_eq!(error.code, ErrorCode::CredentialsMissing);
    }

    #[test]
    fn expired_token_is_rejected_before_request() {
        let now = utc("2026-10-18T12:00:00Z");
        let credentials = |expires_at| Credentials {
            access_token: AccessToken("t".into()),
            expires_at,
        };

        let expired = credentials(Some(now - chrono::Duration::seconds(1)));
        assert_eq!(
            expired.ensure_not_expired(now).unwrap_err().code,
            ErrorCode::TokenExpired
        );
        assert!(
            credentials(Some(now + chrono::Duration::hours(1)))
                .ensure_not_expired(now)
                .is_ok()
        );
        assert!(credentials(None).ensure_not_expired(now).is_ok());
    }

    #[test]
    fn debug_output_hides_token() {
        let credentials = parse_credentials(
            r#"{"claudeAiOauth": {"accessToken": "super-secret", "expiresAt": 1}}"#,
        )
        .unwrap();

        assert!(!format!("{credentials:?}").contains("super-secret"));
    }
}
