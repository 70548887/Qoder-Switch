use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Proxy error: {0}")]
    Proxy(String),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Cert error: {0}")]
    Cert(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Account not found: {0}")]
    AccountNotFound(String),
}

#[derive(Serialize)]
struct SerializedError {
    message: String,
    kind: String,
}

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(err: AppError) -> Self {
        let kind = match &err {
            AppError::Io(_) => "IO",
            AppError::Json(_) => "JSON",
            AppError::Proxy(_) => "Proxy",
            AppError::Auth(_) => "Auth",
            AppError::Cert(_) => "Cert",
            AppError::Config(_) => "Config",
            AppError::AccountNotFound(_) => "AccountNotFound",
        };
        let serialized = SerializedError {
            message: err.to_string(),
            kind: kind.to_string(),
        };
        tauri::ipc::InvokeError::from(
            serde_json::to_string(&serialized).unwrap_or_else(|_| err.to_string())
        )
    }
}

pub type AppResult<T> = Result<T, AppError>;
