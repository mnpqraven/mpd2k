use crate::client::{library::LibraryClient, PlaybackEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};
use strum::{Display, EnumIter};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct AppState {
    // TODO: state for tab
    pub navigation: NavigationState,
    pub library_client: Arc<Mutex<LibraryClient>>,
    pub library_loading: Arc<Mutex<bool>>,
    pub playback_tx: UnboundedSender<PlaybackEvent>,
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
}

pub type Tui = Terminal<CrosstermBackend<Stdout>>;
