use crate::backend::library::{cache::try_load_cache, AudioTrack};
use ratatui::widgets::TableState;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::instrument;

#[derive(Debug)]
pub struct LibraryClient {
    pub audio_tracks: Vec<AudioTrack>,
    pub tui_state: Arc<Mutex<TableState>>,
    pub current_track_duration: Option<Duration>,
    pub volume: f32,
    pub loading: bool,
}

impl LibraryClient {
    pub fn set_loading(&mut self, loading: bool) -> &mut Self {
        self.loading = loading;
        self
    }
}

// volume methods
impl LibraryClient {
    pub fn set_volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;
        self
    }
    pub fn volume_up(&mut self) {
        match self.volume {
            0.95.. => self.volume = 1.0,
            _ => self.volume += 0.05,
        }
    }
    pub fn volume_down(&mut self) {
        match self.volume {
            0.05.. => self.volume -= 0.05,
            _ => self.volume = 0.0,
        }
    }
    pub fn volume_percentage(&self) -> u8 {
        (self.volume * 100.0).round() as u8
    }
}

impl Default for LibraryClient {
    #[instrument(ret)]
    fn default() -> Self {
        Self {
            audio_tracks: try_load_cache().unwrap_or_default(),
            // selected_track_index: Default::default(),
            tui_state: Default::default(),
            loading: false,
            volume: 1.0,
            current_track_duration: None,
        }
    }
}

#[cfg(test)]
mod tests {}
