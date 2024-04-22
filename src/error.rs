#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("This feature is not yet unimplemented")]
    Unimplemented,
    #[error("MPD connection error: {0}")]
    MpdClient(String),
    #[error("MPD server returned (data, error): ({0:?} {1})")]
    MpdProtocolError(Vec<String>, String),
}
