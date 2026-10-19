//! User-facing text in English and Portuguese. Every key must be translated in both
//! tables: the `match` blocks are exhaustive, so a missing translation does not compile.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

pub const LANGUAGE_CHANGED: &str = "language://changed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    En,
    Pt,
}

/// Language chosen in the settings; `Auto` follows the system.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguagePref {
    #[default]
    Auto,
    En,
    Pt,
}

/// What the UI needs to translate and to format dates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageInfo {
    pub language: Language,
    /// BCP 47 tag for date and number formatting: the system locale when it is in the
    /// same language, otherwise a default region for the language.
    pub locale: String,
}

/// Language used by the core for notifications, errors and the tray.
static CURRENT: AtomicU8 = AtomicU8::new(Language::En as u8);

pub fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        x if x == Language::Pt as u8 => Language::Pt,
        _ => Language::En,
    }
}

/// Resolves a preference against the system locale.
pub fn info(pref: LanguagePref) -> LanguageInfo {
    resolve(pref, sys_locale::get_locale().as_deref())
}

/// Makes a preference the language of the core and returns what the UI needs.
pub fn apply(pref: LanguagePref) -> LanguageInfo {
    let info = info(pref);
    CURRENT.store(info.language as u8, Ordering::Relaxed);
    info
}

fn resolve(pref: LanguagePref, system_locale: Option<&str>) -> LanguageInfo {
    let system_language = system_locale.and_then(language_of);
    let language = match pref {
        LanguagePref::Auto => system_language.unwrap_or(Language::En),
        LanguagePref::En => Language::En,
        LanguagePref::Pt => Language::Pt,
    };
    let locale = match system_locale {
        Some(tag) if system_language == Some(language) => normalize_tag(tag),
        _ => match language {
            Language::En => "en-US".into(),
            Language::Pt => "pt-BR".into(),
        },
    };
    LanguageInfo { language, locale }
}

/// Turns POSIX locales like `pt_BR.UTF-8` into BCP 47 tags like `pt-BR`.
fn normalize_tag(tag: &str) -> String {
    tag.split(['.', '@'])
        .next()
        .unwrap_or(tag)
        .replace('_', "-")
}

/// Language of a locale tag, when it is one the app is translated into.
fn language_of(tag: &str) -> Option<Language> {
    let tag = tag.to_ascii_lowercase();
    if tag.starts_with("pt") {
        Some(Language::Pt)
    } else if tag.starts_with("en") {
        Some(Language::En)
    } else {
        None
    }
}

/// Replaces `{name}` placeholders in a translated template.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    args.iter()
        .fold(template.to_owned(), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    // Usage windows
    Session5h,
    Weekly7d,
    // Durations
    LessThanOneMinute,
    // Notifications
    ThresholdBody,
    ResetsInSentence,
    ResetsInTitle,
    CurrentUsage,
    SummaryTitle,
    SummaryReset,
    SummaryEmpty,
    // Claude provider errors
    RenewHint,
    TokenRefused,
    RateLimited,
    UnexpectedStatus,
    CredentialsMissing,
    CredentialsUnreadable,
    CredentialsInvalid,
    TokenExpired,
    UsageFormat,
    UsageInvalidJson,
    UsageNotObject,
    UsageNoWindows,
    NetworkTimeout,
    NetworkFailure,
    // Validation and persistence
    ThresholdRange,
    MinutesMinimum,
    PollIntervalMinimum,
    OpacityRange,
    RuleNotFound,
    SaveFailed,
    // Tray
    TrayOpen,
    TrayOverlay,
    TrayRefresh,
    TrayQuit,
}

impl Language {
    pub fn text(self, key: Key) -> &'static str {
        match self {
            Self::En => en(key),
            Self::Pt => pt(key),
        }
    }
}

/// Shorthand for a text in the current language.
pub fn t(key: Key) -> &'static str {
    current().text(key)
}

fn en(key: Key) -> &'static str {
    use Key::*;
    match key {
        Session5h => "Session (5h)",
        Weekly7d => "Weekly (7 days)",
        LessThanOneMinute => "less than 1 min",
        ThresholdBody => "Reached the {percent}% threshold.{reset}",
        ResetsInSentence => " Resets in {duration}.",
        ResetsInTitle => "{window} resets in {duration}",
        CurrentUsage => "Current usage: {percent}%.",
        SummaryTitle => "Usage summary",
        SummaryReset => " (resets in {duration})",
        SummaryEmpty => "No usage data available right now.",
        RenewHint => "Open Claude Code to renew it.",
        TokenRefused => "Claude token refused (HTTP {status}). {hint}",
        RateLimited => "Usage endpoint rate limit reached (HTTP 429). Retrying on the next poll.",
        UnexpectedStatus => "Unexpected server response (HTTP {status}).",
        CredentialsMissing => "Claude Code credentials not found at {path}. Log in to Claude Code.",
        CredentialsUnreadable => "Could not read the credentials at {path} ({error}).",
        CredentialsInvalid => "Claude Code credentials file has an unexpected format.",
        TokenExpired => "Claude token expired. {hint}",
        UsageFormat => "Usage endpoint response has an unexpected format: {detail}.",
        UsageInvalidJson => "invalid JSON",
        UsageNotObject => "the body is not an object",
        UsageNoWindows => "no known usage window",
        NetworkTimeout => "Timed out while fetching Claude usage.",
        NetworkFailure => "Network error while fetching Claude usage: {error}",
        ThresholdRange => "The threshold must be above 0% and at most 100%.",
        MinutesMinimum => "The number of minutes must be at least 1.",
        PollIntervalMinimum => "The poll interval must be at least {minutes} minute.",
        OpacityRange => "Opacity must be between 0 and 100%.",
        RuleNotFound => "Rule not found.",
        SaveFailed => "Could not save ({key}): {error}",
        TrayOpen => "Open window",
        TrayOverlay => "Overlay",
        TrayRefresh => "Refresh now",
        TrayQuit => "Quit",
    }
}

fn pt(key: Key) -> &'static str {
    use Key::*;
    match key {
        Session5h => "Sessão (5h)",
        Weekly7d => "Semanal (7 dias)",
        LessThanOneMinute => "menos de 1 min",
        ThresholdBody => "Atingiu o limiar de {percent}%.{reset}",
        ResetsInSentence => " Reinicia em {duration}.",
        ResetsInTitle => "{window} reinicia em {duration}",
        CurrentUsage => "Uso atual: {percent}%.",
        SummaryTitle => "Resumo de uso",
        SummaryReset => " (reinicia em {duration})",
        SummaryEmpty => "Nenhum dado de uso disponível no momento.",
        RenewHint => "Abra o Claude Code para renová-lo.",
        TokenRefused => "Token do Claude recusado (HTTP {status}). {hint}",
        RateLimited => {
            "Limite de consultas ao endpoint de uso atingido (HTTP 429). \
             Nova tentativa no próximo ciclo."
        }
        UnexpectedStatus => "Resposta inesperada do servidor (HTTP {status}).",
        CredentialsMissing => {
            "Credenciais do Claude Code não encontradas em {path}. Faça login no Claude Code."
        }
        CredentialsUnreadable => "Não foi possível ler as credenciais em {path} ({error}).",
        CredentialsInvalid => "Arquivo de credenciais do Claude Code em formato inesperado.",
        TokenExpired => "Token do Claude expirado. {hint}",
        UsageFormat => "Resposta do endpoint de uso em formato inesperado: {detail}.",
        UsageInvalidJson => "JSON inválido",
        UsageNotObject => "o corpo não é um objeto",
        UsageNoWindows => "nenhuma janela de uso reconhecida",
        NetworkTimeout => "Tempo esgotado ao consultar o uso do Claude.",
        NetworkFailure => "Falha de rede ao consultar o uso do Claude: {error}",
        ThresholdRange => "O limiar deve ser maior que 0% e no máximo 100%.",
        MinutesMinimum => "O número de minutos deve ser pelo menos 1.",
        PollIntervalMinimum => "O intervalo de consulta deve ser de pelo menos {minutes} minuto.",
        OpacityRange => "A opacidade deve estar entre 0 e 100%.",
        RuleNotFound => "Regra não encontrada.",
        SaveFailed => "Não foi possível salvar ({key}): {error}",
        TrayOpen => "Abrir janela",
        TrayOverlay => "Overlay",
        TrayRefresh => "Atualizar agora",
        TrayQuit => "Sair",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_follows_the_system_language() {
        assert_eq!(
            resolve(LanguagePref::Auto, Some("pt-BR")).language,
            Language::Pt
        );
        assert_eq!(
            resolve(LanguagePref::Auto, Some("pt_PT")).language,
            Language::Pt
        );
        assert_eq!(
            resolve(LanguagePref::Auto, Some("en-GB")).language,
            Language::En
        );
        assert_eq!(
            resolve(LanguagePref::Auto, Some("de-DE")).language,
            Language::En
        );
        assert_eq!(resolve(LanguagePref::Auto, None).language, Language::En);
    }

    #[test]
    fn explicit_choice_overrides_the_system() {
        assert_eq!(
            resolve(LanguagePref::En, Some("pt-BR")).language,
            Language::En
        );
        assert_eq!(
            resolve(LanguagePref::Pt, Some("en-US")).language,
            Language::Pt
        );
    }

    #[test]
    fn locale_keeps_the_system_region_only_for_the_same_language() {
        assert_eq!(resolve(LanguagePref::Auto, Some("pt_PT")).locale, "pt-PT");
        assert_eq!(
            resolve(LanguagePref::Auto, Some("pt_BR.UTF-8")).locale,
            "pt-BR"
        );
        assert_eq!(resolve(LanguagePref::Auto, Some("en-GB")).locale, "en-GB");
        assert_eq!(resolve(LanguagePref::Pt, Some("en-GB")).locale, "pt-BR");
        assert_eq!(resolve(LanguagePref::Auto, Some("de-DE")).locale, "en-US");
    }

    #[test]
    fn fills_placeholders() {
        let text = fill(
            "{window} resets in {duration}",
            &[("window", "Session"), ("duration", "2h")],
        );

        assert_eq!(text, "Session resets in 2h");
    }
}
