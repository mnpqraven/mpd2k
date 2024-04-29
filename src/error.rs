use rodio::decoder::DecoderError;

// TODO: split err
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error("The mutex was poisoned")]
    PoisonError(String),

    #[error(transparent)]
    DecoderError(#[from] DecoderError),

    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),

    #[error(transparent)]
    LibraryMetadata(#[from] audiotags::Error),
    #[error(transparent)]
    LibraryPlayback(#[from] rodio::PlayError),
    // #[error(transparent)]
    // MpdPlayback,
    #[error("This feature is not yet unimplemented")]
    Unimplemented,
    #[error("MPD connection error: {0}")]
    MpdClient(String),
    #[error("MPD server returned (data, error): ({0:?} {1})")]
    MpdProtocol(Vec<String>, String),
    #[error("No dotfile configuration found")]
    NoConfig,
    #[error("Bad dotfile configuration")]
    BadConfig,
    #[error("Not supported for this platform")]
    NotSupported,
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        AppError::PoisonError(err.to_string())
    }
}
