use super::types::*;
use crate::{backend::library::cache::update_cache, client::Playback, dotfile::DotfileSchema};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

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
    pub fn tick(&self) {}

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn update_lib(&mut self) {
        // probably wrong
        let tree_arced = self.library_client.clone();
        let loading_arced = self.library_loading.clone();

        self.toggle_loading();

        tokio::spawn(async move {
            let cfg = DotfileSchema::parse().unwrap();
            // let mut audio_tree = tree_arced.lock().unwrap();
            // TODO: caching
            // audio_tree.audio_tracks = load_all_tracks(&cfg).unwrap();

            // NOTE: caching impl
            let _ = update_cache(&cfg, tree_arced).await.unwrap();
            // if let Ok(tracks) = update_cache(&cfg, tree_arced) {
            //     audio_tree.audio_tracks = tracks;
            // };

            let mut loading = loading_arced.lock().unwrap();
            *loading = !*loading;
        });
    }

    pub fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    pub fn toggle_loading(&mut self) {
        let mut loading = self.library_loading.try_lock().unwrap();
        *loading = !*loading;
    }

    pub fn play(&mut self) {
        let lib_arc = self.library_client.clone();
        tokio::spawn(async move {
            let lib = lib_arc.lock().unwrap();
            let tui_state = lib.tui_state.lock().unwrap();

            let index = tui_state.selected().unwrap();
            let track = lib.audio_tracks.get(index).unwrap();

            drop(tui_state);
            // FIX: this causes blocking
            // till the song is over
            lib.play(Some(track)).unwrap();
        });
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

    pub fn select_prev_track(&mut self) {
        let audio_tree = self.library_client.lock().unwrap();
        let mut tui_state = audio_tree.tui_state.lock().unwrap();
        match tui_state.selected() {
            Some(index) => {
                if index > 1 {
                    tui_state.select(Some(index - 1))
                }
            }
            None => tui_state.select(Some(0)),
        }
    }
}
