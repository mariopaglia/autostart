use serde::{Serialize, Serializer};
use ts_rs::TS;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("profile not found: {0}")]
    ProfileNotFound(String),

    #[error("the last remaining profile cannot be deleted")]
    LastProfile,

    #[error("executable not found: {0}")]
    ExecutableNotFound(String),

    #[error("failed to launch {name}: {reason}")]
    LaunchFailed { name: String, reason: String },

    #[error("process {0} was not detected after launch")]
    ProcessNotDetected(String),

    #[cfg_attr(not(windows), allow(dead_code))]
    #[error("elevation was denied by the user")]
    ElevationDenied,

    #[error("access denied while closing {0}: run AutoStart as administrator")]
    AccessDenied(String),

    #[error("{0} did not close in time")]
    CloseTimedOut(String),

    #[cfg_attr(windows, allow(dead_code))]
    #[error("not supported on this platform: {0}")]
    Unsupported(&'static str),

    #[error("{0}")]
    Tauri(#[from] tauri::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ErrorKind {
    Io,
    Json,
    Validation,
    ProfileNotFound,
    LastProfile,
    ExecutableNotFound,
    LaunchFailed,
    ProcessNotDetected,
    ElevationDenied,
    AccessDenied,
    CloseTimedOut,
    Unsupported,
    Internal,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ErrorPayload {
    pub kind: ErrorKind,
    pub message: String,
}

impl AppError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(_) => ErrorKind::Io,
            Self::Json(_) => ErrorKind::Json,
            Self::Validation(_) => ErrorKind::Validation,
            Self::ProfileNotFound(_) => ErrorKind::ProfileNotFound,
            Self::LastProfile => ErrorKind::LastProfile,
            Self::ExecutableNotFound(_) => ErrorKind::ExecutableNotFound,
            Self::LaunchFailed { .. } => ErrorKind::LaunchFailed,
            Self::ProcessNotDetected(_) => ErrorKind::ProcessNotDetected,
            Self::ElevationDenied => ErrorKind::ElevationDenied,
            Self::AccessDenied(_) => ErrorKind::AccessDenied,
            Self::CloseTimedOut(_) => ErrorKind::CloseTimedOut,
            Self::Unsupported(_) => ErrorKind::Unsupported,
            Self::Tauri(_) => ErrorKind::Internal,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ErrorPayload {
            kind: self.kind(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
