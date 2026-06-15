use thiserror::Error;

/// Centralized application error type.
/// Each variant carries enough context for the frontend to display a meaningful message.
#[derive(Debug, Error)]
pub enum AppError {
    /// Database-level errors: schema, constraint, IO, etc.
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    /// Data not found (card, deck, etc.)
    #[error("未找到: {0}")]
    NotFound(String),

    /// Invalid user input
    #[error("输入无效: {0}")]
    InvalidInput(String),

    /// TTS service errors
    #[error("TTS 服务错误: {0}")]
    Tts(String),

    /// Lock poisoned (shouldn't happen under normal operation)
    #[error("内部锁错误")]
    Lock(String),

    /// I/O errors
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience: convert AppError to the String type used by Tauri commands.
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

/// Shorthand for Result<T, AppError>.
pub type AppResult<T> = Result<T, AppError>;
