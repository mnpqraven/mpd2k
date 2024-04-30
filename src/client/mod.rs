use crate::error::AppError;
use std::{path::PathBuf, time::Duration};

pub mod library;
pub mod mpd;
pub mod events;

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
