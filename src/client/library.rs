use crate::backend::library::{cache::try_load_cache, AudioTrack};
use ratatui::widgets::TableState;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug)]
pub struct LibraryClient {
    pub audio_tracks: Vec<AudioTrack>,
    pub tui_state: Arc<Mutex<TableState>>,
}

impl Default for LibraryClient {
    fn default() -> Self {
        info!("running default");
        Self {
            audio_tracks: try_load_cache().unwrap_or_default(),
            // selected_track_index: Default::default(),
            tui_state: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {}
