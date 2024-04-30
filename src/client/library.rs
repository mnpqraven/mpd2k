use super::{events::PlaybackEvent, ClientKind, PlaybackClient};
use crate::{
    backend::library::{
        cache::{try_load_cache, try_write_cache},
        create_source, load_all_tracks_unhashed, AudioTrack,
    },
    dotfile::DotfileSchema,
    error::AppError,
};
use ratatui::widgets::TableState;
use rodio::Source;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct LibraryClient {
    pub audio_tracks: Vec<AudioTrack>,
    pub current_track: Option<CurrentTrack>,
    pub playback_tx: UnboundedSender<PlaybackEvent>,
    pub volume: f32,
    pub loading: bool,
}
// NOTE: does this really need clone ?
#[derive(Debug, Clone)]
pub struct CurrentTrack {
    pub data: AudioTrack,
    pub duration: Duration,
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

impl LibraryClient {
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self {
            audio_tracks: try_load_cache(DotfileSchema::cache_path().unwrap()).unwrap_or_default(),
            // selected_track_index: Default::default(),
            // tui_state: Default::default(),
            loading: false,
            volume: 1.0,
            current_track: None,
            playback_tx,
        }
    }
}

impl PlaybackClient for LibraryClient {
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError> {
        let track = table_state
            .selected()
            .and_then(|index| self.audio_tracks.get(index))
            .unwrap();

        let source = create_source(track.path.clone()).unwrap();

        self.current_track = Some(CurrentTrack {
            data: track.clone(),
            duration: source.total_duration().unwrap(),
        });

        let _ = self
            .playback_tx
            .send(PlaybackEvent::Play(track.path.clone()));
        Ok(())
    }

    fn kind(&self) -> ClientKind {
        ClientKind::Library
    }

    fn select_next_track(&self, table_state: &mut TableState) {
        let max = self.audio_tracks.len();

        match table_state.selected() {
            Some(index) => {
                if index + 1 < max {
                    table_state.select(Some(index + 1));
                }
            }
            None => table_state.select(Some(0)),
        }
    }

    fn select_prev_track(&self, table_state: &mut TableState) {
        match table_state.selected() {
            Some(index) => {
                if index >= 1 {
                    table_state.select(Some(index - 1))
                }
            }
            None => table_state.select(Some(0)),
        }
    }

    fn pause_unpause(&self) {
        let _ = self.playback_tx.send(PlaybackEvent::Pause);
    }

    fn volume_up(&mut self) {
        let _ = self.playback_tx.send(PlaybackEvent::VolumeDown);
        self.volume_up()
    }

    fn volume_down(&mut self) {
        let _ = self.playback_tx.send(PlaybackEvent::VolumeUp);
        self.volume_down()
    }

    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<LibraryClient>>>) {
        if let Some(self_arc) = self_arc
            && !self.loading
        {
            self.set_loading(true);

            tokio::spawn(async move {
                let cfg = DotfileSchema::parse().unwrap();
                let tracks = load_all_tracks_unhashed(&cfg, self_arc.clone()).unwrap();

                tokio::spawn(async move {
                    let _ = try_write_cache(&DotfileSchema::cache_path().unwrap(), &tracks).await;
                });

                if let Ok(mut lib) = self_arc.lock() {
                    lib.set_loading(false);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {}
