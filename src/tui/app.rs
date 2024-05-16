use crate::client::events::PlaybackToAppEvent;
use crate::client::PlayableClient;
use crate::client::{events::AppToPlaybackEvent, PlaybackClient};
use crate::error::AppError;
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
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::info;

#[derive(Debug)]
pub struct AppState<Client: PlayableClient> {
    pub navigation: NavigationState,
    pub client: PlaybackClient<Client>,
    pub pb_server: UnboundedSender<AppToPlaybackEvent>,
    pub pb_client: UnboundedSender<PlaybackToAppEvent>,
    pub tui_state: TuiState,
    pub exit: bool,
}

#[derive(Debug)]
pub struct TuiState {
    pub playback_table: Arc<Mutex<TableState>>,
    pub library_table: Arc<Mutex<TableState>>,
    pub show_right_sidebar: bool,
    pub show_left_sidebar: bool,
    pub last_album: Option<Arc<str>>,
    pub key_mode: KeyMode,
    pub key_queue: Vec<KeyCode>,
}

#[derive(Debug, Default, PartialEq)]
pub enum KeyMode {
    #[default]
    Normal,
    Multi(Vec<KeyCode>),
    Editing,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            playback_table: Default::default(),
            library_table: Default::default(),
            show_right_sidebar: true,
            show_left_sidebar: true,
            last_album: None,
            key_mode: Default::default(),
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
    LibraryTree,
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
    pub fn new(
        pb_send: UnboundedSender<AppToPlaybackEvent>,
        app_send: UnboundedSender<PlaybackToAppEvent>,
        hash_handle: Handle,
    ) -> Self {
        Self {
            navigation: NavigationState::default(),
            tui_state: Default::default(),
            client: PlaybackClient::new(&pb_send, &app_send, hash_handle),
            exit: false,
            pb_server: pb_send,
            pb_client: app_send,
        }
    }

    pub fn spawn_listener(&self, mut app_rx: UnboundedReceiver<PlaybackToAppEvent>) {
        let _pb_client = self.client.arced();
        tokio::spawn(async move {
            while let Some(message) = app_rx.recv().await {
                match message {
                    PlaybackToAppEvent::CurrentDuration(num) => {
                        info!("from lib thread {}", num);
                        // do smth involving arc
                    }
                }
            }
            Ok::<(), AppError>(())
        });
    }

    /// noop
    pub fn tick(&self) -> Result<(), AppError> {
        Ok(())
    }

    /// sets `exit` flag to `true`, only for the map app's while loop
    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn set_keymode(&mut self, key_mode: KeyMode) {
        self.tui_state.key_mode = key_mode;
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
