use crate::client::library::LibraryClient;
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};
use strum::{Display, EnumIter};

#[derive(Debug)]
pub struct AppState {
    // TODO: state for tab
    pub navigation: NavigationState,
    pub library_client: Arc<Mutex<LibraryClient>>,
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
}

pub type Tui = Terminal<CrosstermBackend<Stdout>>;
