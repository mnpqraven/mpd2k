pub mod event_handlers;
pub mod render;

use crate::{
    backend::library::{load_all_tracks, load_all_tracks_async, AudioTrack},
    dotfile::DotfileSchema,
    error::AppError,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io::{self, Stdout},
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use strum::{Display, EnumIter};
use tracing::info;

#[derive(Debug, Default)]
pub struct StatefulTui {
    // TODO: state for tab
    pub navigation: NavigationState,
    pub audio_tree: Arc<Mutex<AudioTreeState>>,
    loading_lib: Arc<Mutex<bool>>,
    exit: bool,
}

#[derive(Debug, Default)]
pub struct AudioTreeState {
    // TODO: migrate this to albums, or convert function
    pub audio_tracks: Vec<AudioTrack>,
    pub selected_track_index: u32,
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

impl Widget for &StatefulTui {
    /// TODO:
    /// ```
    /// // |----------------------|
    /// // |    navbar   |        |
    /// // |-------------|        |
    /// // |             |  track |
    /// // |   maindex   |  info  |
    /// // |             |        |
    /// // |----------------------|
    /// ```
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let container_ltr = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(area);
        let container_left_ud = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                // 2 for borders + name + prolly shortcut
                Constraint::Max(4),
                // fill rest
                Constraint::Min(10),
            ])
            .split(container_ltr[0]);

        // let navbar = Paragraph::new("navbar").block(Block::new().borders(Borders::all()));
        // let main = Paragraph::new("main").block(Block::new().borders(Borders::all()));

        let loading = self.loading_lib.try_lock().unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let sidebar_right = Paragraph::new(format!("{}\nloading: {}", now, loading))
            .block(Block::new().borders(Borders::all()));

        // NAVBAR
        self.navigation.render(container_left_ud[0], buf);
        // frame.render_widget(navbar, container_left_ud[0]);
        // MAIN BOX
        match self.audio_tree.try_lock() {
            Ok(audio_tree) => audio_tree.render(container_left_ud[1], buf),
            Err(_) => AudioTreeState::default().render(container_left_ud[1], buf),
        };
        // frame.render_widget(main, container_ltr[0]);
        // RIGHT SIDEBAR
        sidebar_right.render(container_ltr[1], buf);
    }
}

impl StatefulTui {
    pub async fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            self.handle_events().await?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        }

        Ok(())
    }

    // TODO: context
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // universal
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('u') => self.update_lib(),
            KeyCode::Char('1') => self.navigate(NavigationRoute::Playback),
            KeyCode::Char('2') => self.navigate(NavigationRoute::Config),
            _ => {}
        }

        // page-dependent
        match self.navigation.current {
            NavigationRoute::Playback => match key_event.code {
                KeyCode::Char('n') => self.select_next_track(),
                KeyCode::Char('p') => self.select_prev_track(),
                _ => {}
            },
            NavigationRoute::Config => {}
        }
    }

    fn exit(&mut self) {
        info!("exit");
        self.exit = true;
    }

    fn update_lib(&mut self) {
        // probably wrong
        let tree_arced = self.audio_tree.clone();

        self.toggle_lib_loading();

        tokio::spawn(async move {
            let cfg = DotfileSchema::parse().unwrap();
            let mut audio_tree = tree_arced.lock().unwrap();
            // TODO: caching
            audio_tree.audio_tracks = load_all_tracks(&cfg).unwrap();
        });
    }

    fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    fn select_next_track(&mut self) {
        // TODO: check last
        let mut audio_tree = self.audio_tree.lock().unwrap();
        audio_tree.selected_track_index += 1;
    }

    fn toggle_lib_loading(&mut self) {
        let mut loading = self.loading_lib.try_lock().unwrap();
        *loading = !*loading;
    }

    fn select_prev_track(&mut self) {
        let mut audio_tree = self.audio_tree.lock().unwrap();
        if audio_tree.selected_track_index > 0 {
            audio_tree.selected_track_index -= 1;
        }
    }
}
