use crate::client::library::LibraryClient;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};
use strum::{Display, EnumIter};

#[derive(Debug, Default)]
pub struct StatefulTui {
    // TODO: state for tab
    pub navigation: NavigationState,
    pub library_tree: Arc<Mutex<LibraryClient>>,
    pub loading_lib: Arc<Mutex<bool>>,
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
