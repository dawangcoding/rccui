use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Serializable error representation for Tauri command responses.
#[derive(Debug, Serialize)]
struct ErrorPayload {
    code: &'static str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let payload = match self {
            AppError::BadRequest(msg) => ErrorPayload {
                code: "BAD_REQUEST",
                message: msg.clone(),
            },
            AppError::NotFound(msg) => ErrorPayload {
                code: "NOT_FOUND",
                message: msg.clone(),
            },
            AppError::Conflict(msg) => ErrorPayload {
                code: "CONFLICT",
                message: msg.clone(),
            },
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                ErrorPayload {
                    code: "INTERNAL",
                    message: "Internal server error".to_string(),
                }
            }
            AppError::Io(e) => {
                tracing::error!("IO error: {e}");
                ErrorPayload {
                    code: "INTERNAL",
                    message: "Internal server error".to_string(),
                }
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {e}");
                ErrorPayload {
                    code: "INTERNAL",
                    message: "Internal server error".to_string(),
                }
            }
        };
        payload.serialize(serializer)
    }
}
