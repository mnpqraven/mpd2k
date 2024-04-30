use super::types::{AppState, NavigationRoute, NavigationState, Tui};
use crate::client::{events::PlaybackEvent, PlaybackClient};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
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

impl<Client: PlaybackClient> AppState<Client> {
    // TODO: generic refactor
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self {
            navigation: NavigationState::default(),
            tui_state: Default::default(),
            library_client: Arc::new(Mutex::new(Client::new(playback_tx))),
            exit: bool::default(),
        }
    }

    pub fn tick(&self) {}

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn update_lib(&mut self) {
        let lib_arced = self.library_client.clone();
        let mut lib = self.library_client.lock().unwrap();
        lib.update_lib(Some(lib_arced));
    }

    pub fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    pub fn play(&mut self) {
        let mut lib = self.library_client.lock().unwrap();
        let table_state = self.tui_state.lock().unwrap();
        let _ = lib.play(&table_state);
    }

    pub fn select_next_track(&mut self) {
        let mut table_state = self.tui_state.lock().unwrap();
        let _ = self
            .library_client
            .lock()
            .map(|lib| lib.select_next_track(&mut table_state));
    }

    pub fn pause_unpause(&mut self) {
        let _ = self.library_client.lock().map(|lib| lib.pause_unpause());
    }

    pub fn select_prev_track(&mut self) {
        let mut tui_state = self.tui_state.lock().unwrap();
        let _ = self
            .library_client
            .lock()
            .map(|lib| lib.select_prev_track(&mut tui_state));
    }

    pub fn volume_down(&mut self) {
        let _ = self.library_client.lock().map(|mut lib| lib.volume_down());
    }

    pub fn volume_up(&mut self) {
        let _ = self.library_client.lock().map(|mut lib| lib.volume_up());
    }
}
