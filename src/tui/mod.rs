pub mod event_handlers;
pub mod playback;

use crate::{
    backend::library::{load_all_tracks, AudioTrack},
    dotfile::DotfileSchema,
    error::AppError,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, Stdout};

#[derive(Debug, Default)]
pub struct StatefulTui {
    // TODO: state for tab
    navigation: NavigationRoute,
    pub audio_tree: AudioTreeState,
    exit: bool,
}

#[derive(Debug, Default)]
pub struct AudioTreeState {
    // TODO: migrate this to albums, or convert function
    pub audio_tracks: Vec<AudioTrack>,
    pub selected_track_index: u32,
}

#[derive(Debug, Default)]
enum NavigationRoute {
    #[default]
    Playback,
    Config,
}

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

impl StatefulTui {
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
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                let size = frame.size();

                let container_ltr = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
                    .split(size);
                let container_left_ud = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        // 2 for borders + name + prolly shortcut
                        Constraint::Max(4),
                        // fill rest
                        Constraint::Min(10),
                    ])
                    .split(container_ltr[0]);

                let navbar = Paragraph::new("navbar").block(Block::new().borders(Borders::all()));

                let sidebar_right =
                    Paragraph::new("sidebar_right").block(Block::new().borders(Borders::all()));

                // NAVBAR
                frame.render_widget(navbar, container_left_ud[0]);
                // MAIN BOX
                frame.render_widget(&self.audio_tree, container_left_ud[1]);
                // RIGHT SIDEBAR
                frame.render_widget(sidebar_right, container_ltr[1]);
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
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
            _ => {}
        }

        // page-dependent
        match self.navigation {
            NavigationRoute::Playback => match key_event.code {
                KeyCode::Char('n') => self.select_next_track(),
                KeyCode::Char('p') => self.select_prev_track(),
                _ => {}
            },
            NavigationRoute::Config => {}
        }
    }

    pub fn load_all(&mut self) -> Result<&mut Self, AppError> {
        let cfg = DotfileSchema::parse().unwrap();
        self.audio_tree.audio_tracks = load_all_tracks(&cfg)?;
        Ok(self)
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next_track(&mut self) {
        // TODO: check last
        self.audio_tree.selected_track_index += 1;
    }
    fn select_prev_track(&mut self) {
        if self.audio_tree.selected_track_index > 0 {
            self.audio_tree.selected_track_index -= 1;
        }
    }
}
