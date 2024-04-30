use crate::error::AppError;
use std::path::PathBuf;

pub mod events;
pub mod library;
pub mod mpd;

pub enum ClientKind {
    Library,
    Mpd,
}

pub trait PlayableAudio {
    fn path(&self) -> PathBuf;
}

pub trait PlaybackClient {
    fn play(&self) -> Result<(), AppError>;
    // TODO: pause
    // stop
    // queue next
    // seek
    fn kind(&self) -> ClientKind;
}
