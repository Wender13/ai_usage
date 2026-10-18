use serde::Serialize;

/// Machine-readable error category, used by the UI to pick how to present the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    CredentialsMissing,
    CredentialsInvalid,
    TokenExpired,
    Unauthorized,
    RateLimited,
    Network,
    Http,
    Parse,
}

/// Error reported by a provider. The message is shown in the UI as is,
/// so it must never contain credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("{message}")]
pub struct ProviderError {
    pub code: ErrorCode,
    pub message: String,
}

impl ProviderError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
