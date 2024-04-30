use crate::error::AppError;
use std::{path::PathBuf, time::Duration};

pub mod library;
pub mod mpd;
pub mod events;

pub trait PlayableAudio {
    fn path(&self) -> PathBuf;
}

pub trait Playback {
    fn play(&self, audio: Option<impl PlayableAudio>) -> Result<(), AppError>;
    // TODO: pause
    // stop
    // queue next
    // seek
    fn _play(audio: Option<impl PlayableAudio>) -> Result<(), AppError>;
}

pub trait Toggle {
    fn fade_in_out(&self, duration: Option<Duration>) -> Result<(), AppError>;
    // TODO: set volume
    // set gain
    // set sample rate (only lib ?)
}
