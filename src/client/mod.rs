use self::{events::PlaybackEvent, library::CurrentTrack};
use crate::{backend::library::AudioTrack, error::AppError};
use ratatui::widgets::TableState;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

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
    fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self;
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError>;

    fn current_track(&self) -> Option<&CurrentTrack>;

    fn loading(&self) -> bool;

    fn audio_tracks(&self) -> &[AudioTrack];

    fn select_next_track(&self, table_state: &mut TableState);
    fn select_prev_track(&self, table_state: &mut TableState);
    fn select_last_track(&self, table_state: &mut TableState);
    fn pause_unpause(&self);
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<Self>>>);

    fn volume_percentage(&self) -> u8;
    fn volume_up(&mut self);
    fn volume_down(&mut self);

    // TODO:
    // stop
    // queue next
    // seek
    fn kind(&self) -> ClientKind;
}
