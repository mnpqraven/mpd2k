use crate::client::events::PlaybackEvent;
use crate::client::PlaybackClient;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::io::{self, stdout};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};
use strum::{Display, EnumIter};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct AppState<Client: PlaybackClient> {
    // TODO: state for tab
    pub navigation: NavigationState,
    // TODO: make wrapper struct for references, fn impl and inner
    pub library_client: Arc<Mutex<Client>>,
    pub tui_state: Arc<Mutex<TableState>>,
    pub exit: bool,
}

#[derive(Debug, Default)]
pub struct NavigationState {
    pub current: NavigationRoute,
}

#[derive(Debug, Default, EnumIter, Display, PartialEq, Eq)]
pub enum NavigationRoute {
    #[default]
    Playback,
    Config,
    Help,
}

/// Initialize the terminal
pub fn init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
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
    // TODO: doc
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self {
            navigation: NavigationState::default(),
            tui_state: Default::default(),
            library_client: Arc::new(Mutex::new(Client::new(playback_tx))),
            exit: bool::default(),
        }
    }

    /// noop
    pub fn tick(&self) {}

    /// sets `exit` flag to `true`, only for the map app's while loop
    pub fn exit(&mut self) {
        self.exit = true;
    }

    /// Triggers the update on the audio list,
    ///
    /// For library the directory is fully loaded unhashed then a
    /// hashing worker is queued in the background
    pub fn update_lib(&mut self) {
        let lib_arced = self.library_client.clone();
        let mut lib = self.library_client.lock().unwrap();
        lib.update_lib(Some(lib_arced));
    }

    /// records the previous keys for multi-key command (can be either modifier
    /// + key or multi keys like `gg`)
    pub fn set_multi_key(&mut self, _key: KeyCode) {
        todo!()
    }

    /// clears the multi key registry
    pub fn flush_multi_key(&mut self) {
        // TODO: non empty bucket
        if true {
            todo!()
        }
    }

    /// Change the navigation route of the app
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

    pub fn select_last_track(&mut self) {
        let mut table_state = self.tui_state.lock().unwrap();
        let _ = self
            .library_client
            .lock()
            .map(|lib| lib.select_last_track(&mut table_state));
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
