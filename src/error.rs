
// TODO: split err
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),

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
