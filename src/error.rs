use rodio::decoder::DecoderError;

// TODO: split err
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("bad csv entry")]
    CsvParse,

    #[error("The mutex was poisoned")]
    PoisonError(String),
    #[error("Failed sending message to thread")]
    SendError(String),

    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),

    #[error(transparent)]
    LibraryClient(#[from] LibraryError),
    #[error(transparent)]
    MpdClient(#[from] MpdError),

    #[error("This feature is not yet unimplemented")]
    Unimplemented,
    #[error("No dotfile configuration found")]
    NoConfig,
    #[error("Bad dotfile configuration")]
    BadConfig,
    #[error("Not supported for this platform")]
    NotSupported,

    #[error("Bad unwrap: {0:?}")]
    BadUnwrap(Option<String>),
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error(transparent)]
    LibraryMetadata(#[from] audiotags::Error),
    #[error(transparent)]
    LibraryPlayback(#[from] rodio::PlayError),
    #[error(transparent)]
    DecoderError(#[from] DecoderError),
}

#[derive(Debug, thiserror::Error)]
pub enum MpdError {
    #[error("MPD connection error: {0}")]
    MpdClient(String),
    #[error("MPD server returned (data, error): ({0:?} {1})")]
    MpdProtocol(Vec<String>, String),
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        AppError::PoisonError(err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AppError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        AppError::SendError(err.to_string())
    }
}
