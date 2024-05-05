use super::render::image::ImageState;
use crate::client::PlayableClient;
use crate::client::{events::PlaybackEvent, PlaybackClient};
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
pub struct AppState<Client: PlayableClient> {
    pub navigation: NavigationState,
    pub client: PlaybackClient<Client>,
    pub tui_state: TuiState,
    pub exit: bool,
}

#[derive(Debug)]
pub struct TuiState {
    pub playback_table: Arc<Mutex<TableState>>,
    pub show_right_sidebar: bool,
    pub show_left_sidebar: bool,
    pub image: Arc<Mutex<ImageState>>,
    pub last_album: Option<String>,
    // won't work with normal mutating vec
    pub key_queue: Vec<KeyCode>,
}
impl Default for TuiState {
    fn default() -> Self {
        Self {
            playback_table: Default::default(),
            show_right_sidebar: true,
            show_left_sidebar: true,
            image: Default::default(),
            last_album: None,
            key_queue: Vec::new(),
        }
    }
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

impl<Client: PlayableClient> AppState<Client> {
    // TODO: generic refactor
    // TODO: doc
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        Self {
            navigation: NavigationState::default(),
            tui_state: Default::default(),
            client: PlaybackClient::new(playback_tx),
            exit: bool::default(),
        }
    }

    /// noop
    pub fn tick(&self) {}

    /// sets `exit` flag to `true`, only for the map app's while loop
    pub fn exit(&mut self) {
        self.exit = true;
    }

    /// records the previous keys for multi-key command (can be either modifier
    /// + key or multi keys like `gg`)
    pub fn set_multi_key(&mut self, key: KeyCode) {
        self.tui_state.key_queue.push(key);
    }

    pub fn is_multi_key(&self, keys: impl AsRef<[KeyCode]> + std::fmt::Debug) -> bool {
        self.tui_state.key_queue == keys.as_ref()
    }

    /// clears the multi key registry
    pub fn flush_multi_key(&mut self) {
        self.tui_state.key_queue.clear()
    }

    /// Change the navigation route of the app
    pub fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }
}
