use ratatui::widgets::TableState;

use crate::error::AppError;
use std::{path::PathBuf, sync::{Arc, Mutex}};

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
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError>;
    fn select_next_track(&self, table_state: &mut TableState);
    fn select_prev_track(&self, table_state: &mut TableState);
    fn pause_unpause(&self);
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    /// self_arc: the arc of this client
    /// we need this for updating songs on a background thread
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<Self>>>);

    // TODO:
    // stop
    // queue next
    // seek
    fn kind(&self) -> ClientKind;
}
