use super::{
    events::{AppToPlaybackEvent, PlaybackToAppEvent},
    ClientKind, PlayableClient,
};
use crate::{
    backend::library::{
        create_source,
        expr_mod::{hash_cleanup, load_all_tracks_expr, load_hash_expr},
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
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot,
};
use tracing::{info, instrument};

impl PlayableClient for LibraryClient {
    fn new(
        playback_tx: UnboundedSender<AppToPlaybackEvent>,
        // playback_rx: UnboundedReceiver<PlaybackToAppEvent>,
    ) -> Self {
        Self::new(playback_tx).0
    }

    #[instrument(skip(self))]
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError> {
        let some_track = table_state
            .selected()
            .and_then(|index| self.audio_tracks().get(index).copied().cloned());

        if let Some(track) = some_track {
            info!(?track);

            let source = create_source(track.path.as_ref()).unwrap();

            self.current_track = Some(CurrentTrack {
                data: track.clone(),
                duration: source.total_duration().unwrap_or_default(),
            });

            // the shuffle list is randomized on every hard play
            match (self.repeat, self.shuffle) {
                (_, true) => {
                    let q = self.generate_random_queue()?;
                    let cmds = [
                        AppToPlaybackEvent::SetQueue([track].into()),
                        AppToPlaybackEvent::AppendQueue(q),
                        AppToPlaybackEvent::Play,
                    ];
                    for cmd in cmds {
                        self.playback_tx.send(cmd)?;
                    }
                }
                (_, false) => {
                    let cmds = [
                        AppToPlaybackEvent::SetQueue([track].into()),
                        // TODO: queue album by album
                        AppToPlaybackEvent::Play,
                    ];
                    for cmd in cmds {
                        self.playback_tx.send(cmd)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn kind(&self) -> ClientKind {
        ClientKind::Library
    }

    fn select_next_track(&self, table_state: &mut TuiState) -> Result<(), AppError> {
        let mut table_state = table_state.playback_table.lock().unwrap();
        let max = self.audio_tracks().len();

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
        let max = self.audio_tracks().len();
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
                if let Some(track) = self.audio_tracks().get(index) {
                    if track.album != tui_state.last_album {
                        if let (Ok(mut image), Some(p)) =
                            (tui_state.image.lock(), track.try_cover_path())
                        {
                            image.update(p);
                        }
                    }
                    tui_state.last_album.clone_from(&track.album);
                }
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
        let _ = self.playback_tx.send(AppToPlaybackEvent::TogglePause);
    }

    fn volume_up(&mut self) {
        let _ = self.playback_tx.send(AppToPlaybackEvent::VolumeUp);
        match self.volume {
            0.95.. => self.volume = 1.0,
            _ => self.volume += 0.05,
        }
    }

    fn volume_down(&mut self) {
        // FIX: ERR here
        match self.playback_tx.send(AppToPlaybackEvent::VolumeDown) {
            Ok(_) => {}
            Err(e) => {
                info!(?e);
            }
        }
        match self.volume {
            0.05.. => self.volume -= 0.05,
            _ => self.volume = 0.0,
        }
    }

    fn loading(&self) -> bool {
        self.loading
    }

    fn audio_tracks(&self) -> Vec<&AudioTrack> {
        self.albums().values().flatten().collect()
    }

    fn albums(&self) -> &BTreeMap<AlbumMeta, Vec<AudioTrack>> {
        &self.albums
    }

    /// TODO: impl hash compare
    /// track list also need hash sort and dedup
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<LibraryClient>>>, _hard_update: bool) {
        if let Some(self_arc) = self_arc
            && !self.loading
        {
            self.set_loading(true);
            let handle = self.hashing_rt.handle();
            let hash_handle = handle.clone();
            let hash_handle_2nd = handle.clone();
            let self_arc_2nd = self_arc.clone();

            let (tx, mut rx) = oneshot::channel();

            self.hashing_rt.spawn(async move {
                let now = Instant::now();
                info!("inside hashing thread");

                let lib_root = DotfileSchema::parse()?.library_root()?;
                { // OLD CODE
                     // load_all_tracks_raw(lib_root, self_arc.clone(), hard_update).await?;
                     // inject_metadata(self_arc.clone(), hash_handle.clone()).await?;
                     // inject_hash(self_arc.clone(), hash_handle.clone(), true).await?;
                     // load_albums(self_arc.clone())?;
                }

                // NEW CODE USING LOADING THEN ADDING TO ALBUM BTREE
                load_all_tracks_expr(&lib_root, self_arc.clone(), &hash_handle.clone()).await?;

                if let Ok(mut lib) = self_arc.lock() {
                    // TODO: check for index drift
                    lib.set_loading(false);
                }

                let elapsed = now.elapsed().as_millis();
                if tx.send(true).is_err() {
                    info!("oneshot channel dropped");
                }
                info!("hashing_rt total load: {elapsed} s");
                Ok::<(), AppError>(())
            });

            // FIX: this cause blocking and delay the play message sent to sink
            // just that the message is very delayed during hashing/updating
            self.hashing_rt.spawn(async move {
                info!("inside 2nd hashing thread");
                if let Ok(true) = rx.await {
                    info!("begin hashing update");
                    // FIX: this blocks playback state
                    load_hash_expr(self_arc_2nd.clone(), &hash_handle_2nd.clone()).await?;
                    hash_cleanup(self_arc_2nd.clone())?;
                }
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

        let elapsed = now.elapsed().as_millis();
        info!(elapsed);
        Ok(q_arced)
    }

    fn get_play(&self) -> bool {
        // TODO: talk between threads
        // self.playback_tx
        //     .send(AppToPlaybackEvent::PlayStatus)
        //     .unwrap();
        true
    }
}
