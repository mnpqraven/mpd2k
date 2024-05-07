use super::{events::PlaybackEvent, ClientKind, PlayableClient};
use crate::{
    backend::library::{
        cache::inject_hash,
        create_source, inject_metadata, load_albums, load_all_tracks_raw,
        types::{AlbumMeta, AudioTrack, CurrentTrack, LibraryClient, RepeatMode},
    },
    dotfile::DotfileSchema,
    error::AppError,
    tui::app::TuiState,
};
use rand::seq::SliceRandom;
use ratatui::widgets::TableState;
use rodio::Source;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, instrument};

impl PlayableClient for LibraryClient {
    fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self::new(playback_tx)
    }

    #[instrument(skip(self))]
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError> {
        let track = table_state
            .selected()
            .and_then(|index| self.audio_tracks.get(index))
            .unwrap();
        info!(?track);

        let source = create_source(track.path.as_ref()).unwrap();

        self.current_track = Some(CurrentTrack {
            data: track.clone(),
            duration: source.total_duration().unwrap_or_default(),
        });

        let _ = self
            .playback_tx
            .send(PlaybackEvent::Play(track.path.to_string(), false));

        // the shuffle list is randomized on every hard play
        match (self.repeat, self.shuffle) {
            (_, true) => {
                self.generate_random_queue()?;
                self.playback_tx.send(PlaybackEvent::OnlyPlay)?;
            }
            (_, false) => {
                self.playback_tx.send(PlaybackEvent::OnlyPlay)?;
            }
        }
        Ok(())
    }

    fn kind(&self) -> ClientKind {
        ClientKind::Library
    }

    fn select_next_track(&self, table_state: &mut TuiState) -> Result<(), AppError> {
        let mut table_state = table_state.playback_table.lock().unwrap();
        let max = self.audio_tracks.len();

        match table_state.selected() {
            Some(index) => {
                if index + 1 < max {
                    table_state.select(Some(index + 1));
                }
            }
            None => table_state.select(Some(0)),
        }
        Ok(())
    }

    fn select_prev_track(&self, tui_state: &mut TuiState) -> Result<(), AppError> {
        let mut table_state = tui_state.playback_table.lock()?;
        match table_state.selected() {
            Some(index) => {
                if index >= 1 {
                    table_state.select(Some(index - 1))
                }
            }
            None => table_state.select(Some(0)),
        }
        Ok(())
    }

    fn select_first_track(&self, tui_state: &mut TuiState) -> Result<(), AppError> {
        let mut table_state = tui_state.playback_table.lock()?;
        table_state.select(Some(0));
        Ok(())
    }

    fn select_last_track(&self, tui_state: &mut TuiState) -> Result<(), AppError> {
        let mut table_state = tui_state.playback_table.lock()?;
        let max = self.audio_tracks.len();
        table_state.select(Some(max - 1));
        Ok(())
    }

    fn check_image(&self, tui_state: &mut TuiState) -> Result<(), AppError> {
        let table_state = tui_state.playback_table.lock()?;
        let idx = table_state.selected();
        let img_state = tui_state.image.lock().map(|e| e.0.clone())?;
        match (idx, &img_state) {
            (Some(index), _) => {
                // safe unwrap
                let track = self.audio_tracks.get(index).unwrap();
                if track.album != tui_state.last_album {
                    if let (Ok(mut image), Some(p)) =
                        (tui_state.image.lock(), track.try_cover_path())
                    {
                        image.update(p);
                    }
                }
                tui_state.last_album.clone_from(&track.album);
            }
            (None, Some(_)) => {
                if let Ok(mut image) = tui_state.image.lock() {
                    image.unset();
                }
            }
            (None, None) => {}
        }
        Ok(())
    }

    fn pause_unpause(&self) {
        let _ = self.playback_tx.send(PlaybackEvent::Pause);
    }

    fn volume_up(&mut self) {
        let _ = self.playback_tx.send(PlaybackEvent::VolumeUp);
        match self.volume {
            0.95.. => self.volume = 1.0,
            _ => self.volume += 0.05,
        }
    }

    fn volume_down(&mut self) {
        let _ = self.playback_tx.send(PlaybackEvent::VolumeDown);
        match self.volume {
            0.05.. => self.volume -= 0.05,
            _ => self.volume = 0.0,
        }
    }

    fn loading(&self) -> bool {
        self.loading
    }

    fn audio_tracks(&self) -> Vec<&AudioTrack> {
        let t: Vec<&AudioTrack> = self.albums().values().flatten().collect();
        t
    }

    fn albums(&self) -> &BTreeMap<AlbumMeta, Vec<AudioTrack>> {
        &self.albums
    }

    /// TODO: impl hash compare
    /// track list also need hash sort and dedup
    #[instrument(skip_all)]
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<LibraryClient>>>, hard_update: bool) {
        if let Some(self_arc) = self_arc
            && !self.loading
        {
            self.set_loading(true);
            let handle = self.hashing_rt.handle();
            let hash_handle = handle.clone();

            self.hashing_rt.spawn(async move {
                let now = Instant::now();

                let lib_root = DotfileSchema::parse()?.library_root()?;
                load_all_tracks_raw(lib_root, self_arc.clone(), hard_update).await?;
                inject_metadata(self_arc.clone(), hash_handle.clone()).await?;
                inject_hash(self_arc.clone(), hash_handle.clone(), true).await?;
                load_albums(self_arc.clone())?;

                if let Ok(mut lib) = self_arc.lock() {
                    lib.set_loading(false);
                }

                let elapsed = now.elapsed().as_millis();
                info!("hashing_rt total load: {elapsed} s");
                Ok::<(), AppError>(())
            });
        }
    }

    fn current_track(&self) -> Option<&CurrentTrack> {
        self.current_track.as_ref()
    }

    fn volume_percentage(&self) -> u8 {
        (self.volume * 100.0).round() as u8
    }

    fn cleanup(self) {
        self.hashing_rt.shutdown_background();
    }

    fn cycle_repeat(&mut self) {
        self.repeat = match self.repeat {
            RepeatMode::Off => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::Off,
        };
    }

    fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }
    fn get_repeat(&self) -> RepeatMode {
        self.repeat
    }
    fn get_shuffle(&self) -> bool {
        self.shuffle
    }

    #[instrument(skip_all)]
    fn generate_random_queue(&self) -> Result<Arc<[AudioTrack]>, AppError> {
        let now = Instant::now();
        let mut rng = &mut rand::thread_rng();
        let q: Vec<AudioTrack> = self
            .audio_tracks()
            .choose_multiple(&mut rng, 20)
            .copied()
            .cloned()
            .collect();

        let q_arced: Arc<[AudioTrack]> = Arc::from(q);
        // TODO: safe unwrap
        self.playback_tx
            .send(PlaybackEvent::AppendQueue(q_arced.clone()))?;

        let elapsed = now.elapsed().as_millis();
        info!(elapsed);
        Ok(q_arced)
    }
}
