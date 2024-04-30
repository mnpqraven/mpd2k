use super::types::*;
use crate::{
    backend::library::{cache::try_write_cache, create_source, load_all_tracks_unhashed},
    client::{
        events::PlaybackEvent,
        library::{CurrentTrack, LibraryClient},
    },
    dotfile::DotfileSchema,
};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use rodio::Source;
use std::{
    io::{self, stdout},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// clean up terminal to previous state before exiting
pub fn teardown() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

impl AppState {
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self {
            // playback_tx: playback_tx.clone(),
            navigation: NavigationState::default(),
            library_client: Arc::new(Mutex::new(LibraryClient::new(playback_tx))),
            exit: bool::default(),
        }
    }

    pub fn tick(&self) {}

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn update_lib(&mut self) {
        // probably wrong
        let tree_arced = self.library_client.clone();
        let tree_arced2 = self.library_client.clone();
        let mut guard = tree_arced2.lock().unwrap();
        if !guard.loading {
            guard.set_loading(true);

            tokio::spawn(async move {
                let cfg = DotfileSchema::parse().unwrap();
                // let _ = update_cache(&cfg, tree_arced.clone()).await.unwrap();
                let tracks = load_all_tracks_unhashed(&cfg, tree_arced.clone()).unwrap();

                tokio::spawn(async move {
                    let _ = try_write_cache(&DotfileSchema::cache_path().unwrap(), &tracks).await;
                });

                let mut lib = tree_arced.lock().unwrap();
                lib.set_loading(false);
            });
        }
    }

    pub fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    pub fn play(&mut self) {
        let lib = self.library_client.lock().unwrap();

        let tui_state = lib.tui_state.lock().unwrap();
        let index = tui_state.selected().unwrap();

        let track = lib.audio_tracks.get(index).unwrap();

        drop(tui_state);

        let mut lib = self.library_client.lock().unwrap();
        let source = create_source(track.path.clone()).unwrap();

        lib.current_track = Some(CurrentTrack {
            data: track.clone(),
            duration: source.total_duration().unwrap(),
        });

        let _ = lib
            .playback_tx
            .send(PlaybackEvent::Play(track.path.clone()));
    }

    pub fn select_next_track(&mut self) {
        // TODO: check last
        let audio_tree = self.library_client.lock().unwrap();
        let max = audio_tree.audio_tracks.len();
        let mut tui_state = audio_tree.tui_state.lock().unwrap();
        match tui_state.selected() {
            Some(index) => {
                if index + 1 < max {
                    tui_state.select(Some(index + 1));
                }
            }
            None => tui_state.select(Some(0)),
        }
    }

    pub fn pause_unpause(&mut self) {
        let _ = self
            .library_client
            .lock()
            .map(|lib| lib.playback_tx.send(PlaybackEvent::Pause));
    }

    pub fn select_prev_track(&mut self) {
        let audio_tree = self.library_client.lock().unwrap();
        let mut tui_state = audio_tree.tui_state.lock().unwrap();
        match tui_state.selected() {
            Some(index) => {
                if index >= 1 {
                    tui_state.select(Some(index - 1))
                }
            }
            None => tui_state.select(Some(0)),
        }
    }

    pub fn volume_down(&mut self) {
        let mut audio_tree = self.library_client.lock().unwrap();
        audio_tree.volume_down();

        let _ = audio_tree.playback_tx.send(PlaybackEvent::VolumeDown);
    }
    pub fn volume_up(&mut self) {
        let mut audio_tree = self.library_client.lock().unwrap();
        audio_tree.volume_up();

        let _ = audio_tree.playback_tx.send(PlaybackEvent::VolumeUp);
    }
}
