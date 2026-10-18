use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Claude,
}

impl ProviderId {
    /// Same value as the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowKind {
    #[serde(rename = "rolling_5h")]
    Rolling5h,
    #[serde(rename = "rolling_7d")]
    Rolling7d,
}

impl WindowKind {
    /// Same value as the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rolling5h => "rolling_5h",
            Self::Rolling7d => "rolling_7d",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Rolling5h => "Sessão (5h)",
            Self::Rolling7d => "Semanal (7 dias)",
        }
    }
}

/// One usage limit window of a provider.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub provider: ProviderId,
    pub kind: WindowKind,
    /// Percentage of the limit used, from 0 to 100.
    pub utilization: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

impl UsageWindow {
    /// Identifies the window across polls, e.g. `claude:rolling_5h`.
    pub fn key(&self) -> String {
        format!("{}:{}", self.provider.as_str(), self.kind.as_str())
    }

    pub fn label(&self) -> String {
        format!("{} · {}", self.provider.label(), self.kind.label())
    }
}

/// Latest known state of a provider. On error, the windows from the last
/// successful fetch are kept so the UI can show them as stale.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub provider: ProviderId,
    pub windows: Vec<UsageWindow>,
    pub error: Option<ProviderError>,
    pub last_success_at: Option<DateTime<Utc>>,
}

impl ProviderStatus {
    pub fn new(provider: ProviderId) -> Self {
        Self {
            provider,
            windows: Vec::new(),
            error: None,
            last_success_at: None,
        }
    }

    pub fn record(&mut self, result: Result<Vec<UsageWindow>, ProviderError>, now: DateTime<Utc>) {
        match result {
            Ok(windows) => {
                self.windows = windows;
                self.error = None;
                self.last_success_at = Some(now);
            }
            Err(error) => self.error = Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    /// When the last poll finished; `None` until the first poll completes.
    pub fetched_at: Option<DateTime<Utc>>,
    pub providers: Vec<ProviderStatus>,
}

impl UsageSnapshot {
    pub fn new(providers: impl IntoIterator<Item = ProviderId>) -> Self {
        Self {
            fetched_at: None,
            providers: providers.into_iter().map(ProviderStatus::new).collect(),
        }
    }

    pub fn record(
        &mut self,
        provider: ProviderId,
        result: Result<Vec<UsageWindow>, ProviderError>,
        now: DateTime<Utc>,
    ) {
        if let Some(status) = self.providers.iter_mut().find(|s| s.provider == provider) {
            status.record(result, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    fn window(utilization: f64) -> UsageWindow {
        UsageWindow {
            provider: ProviderId::Claude,
            kind: WindowKind::Rolling5h,
            utilization,
            resets_at: None,
        }
    }

    #[test]
    fn as_str_matches_serialized_form() {
        let provider = ProviderId::Claude;
        assert_eq!(serde_json::to_value(provider).unwrap(), provider.as_str());
        for kind in [WindowKind::Rolling5h, WindowKind::Rolling7d] {
            assert_eq!(serde_json::to_value(kind).unwrap(), kind.as_str());
        }
    }

    #[test]
    fn success_replaces_windows_and_clears_error() {
        let now = Utc::now();
        let mut status = ProviderStatus::new(ProviderId::Claude);
        status.error = Some(ProviderError::new(ErrorCode::Network, "offline"));

        status.record(Ok(vec![window(10.0)]), now);

        assert_eq!(status.windows, vec![window(10.0)]);
        assert_eq!(status.error, None);
        assert_eq!(status.last_success_at, Some(now));
    }

    #[test]
    fn error_keeps_last_windows() {
        let first = Utc::now();
        let mut status = ProviderStatus::new(ProviderId::Claude);
        status.record(Ok(vec![window(10.0)]), first);

        let error = ProviderError::new(ErrorCode::TokenExpired, "expired");
        status.record(Err(error.clone()), first + chrono::Duration::minutes(3));

        assert_eq!(status.windows, vec![window(10.0)]);
        assert_eq!(status.error, Some(error));
        assert_eq!(status.last_success_at, Some(first));
    }
}
