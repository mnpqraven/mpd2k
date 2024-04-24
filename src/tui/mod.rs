pub mod event_handlers;
pub mod render;

use crate::{
    backend::library::{try_load_cache, update_cache, AudioTrack},
    dotfile::DotfileSchema,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, StatefulWidget, TableState, Widget},
};
use std::{
    io::{self, Stdout},
    sync::{Arc, Mutex},
};
use strum::{Display, EnumIter};

#[derive(Debug, Default)]
pub struct StatefulTui {
    // TODO: state for tab
    pub navigation: NavigationState,
    pub audio_tree: Arc<Mutex<AudioTreeState>>,
    loading_lib: Arc<Mutex<bool>>,
    exit: bool,
}

#[derive(Debug)]
pub struct AudioTreeState {
    // TODO: migrate this to albums, or convert function
    pub audio_tracks: Vec<AudioTrack>,
    pub selected_track_index: u32,
    pub tui_state: Arc<Mutex<TableState>>,
}
impl Default for AudioTreeState {
    fn default() -> Self {
        Self {
            audio_tracks: try_load_cache().unwrap(),
            selected_track_index: Default::default(),
            tui_state: Default::default(),
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
                Constraint::Length(3),
                // fill rest
                Constraint::Min(10),
            ])
            .split(container_ltr[0]);

        let sidebar_right =
            Paragraph::new("sidebar right").block(Block::new().borders(Borders::all()));

        // NAVBAR
        self.navigation.render(container_left_ud[0], buf);
        // frame.render_widget(navbar, container_left_ud[0]);
        // MAIN BOX
        let mainbox_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(container_left_ud[1]);
        let mainbox_left = mainbox_layout[0];
        let mainbox_right = mainbox_layout[1];

        // mainbox_left_component.render(area, buf)

        match self.audio_tree.try_lock() {
            Ok(audio_tree) if audio_tree.tui_state.try_lock().is_ok() => {
                let get = audio_tree
                    .audio_tracks
                    .get(audio_tree.selected_track_index as usize);
                let mainbox_left_component = main_left(get);
                mainbox_left_component.render(mainbox_left, buf);

                let mut state = audio_tree.tui_state.try_lock().unwrap();
                audio_tree.render(mainbox_right, buf, &mut state)
            }
            _ => {
                let mut empty = TableState::default();
                AudioTreeState::default().render(mainbox_right, buf, &mut empty);
            }
        };
        // frame.render_widget(main, container_ltr[0]);
        // RIGHT SIDEBAR
        sidebar_right.render(container_ltr[1], buf);
    }
}

fn main_left<'a>(data: Option<&AudioTrack>) -> Paragraph<'a> {
    let block = Block::new().borders(Borders::all());
    match data {
        None => Paragraph::default().block(block),
        Some(data) => {
            let text = vec![
                Line::from(format!("Album: {}", data.album.clone().unwrap_or_default())),
                Line::from(format!(
                    "Album Artist: {}",
                    data.album_artist.clone().unwrap_or_default()
                )),
                Line::from("Year: 2024"),
            ];
            Paragraph::new(text).block(block)
        }
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
            KeyCode::Char('u') => {
                if self.loading_lib.try_lock().is_ok_and(|loading| !*loading) {
                    self.update_lib()
                }
            }

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
        self.exit = true;
    }

    fn update_lib(&mut self) {
        // probably wrong
        let tree_arced = self.audio_tree.clone();
        let loading_arced = self.loading_lib.clone();

        self.toggle_lib_loading();

        tokio::spawn(async move {
            let cfg = DotfileSchema::parse().unwrap();
            let mut audio_tree = tree_arced.lock().unwrap();
            // TODO: caching
            // audio_tree.audio_tracks = load_all_tracks(&cfg).unwrap();

            // NOTE: caching impl
            if let Ok(tracks) = update_cache(&cfg) {
                audio_tree.audio_tracks = tracks;
            };

            let mut loading = loading_arced.lock().unwrap();
            *loading = !*loading;
        });
    }

    fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    fn toggle_lib_loading(&mut self) {
        let mut loading = self.loading_lib.try_lock().unwrap();
        *loading = !*loading;
    }

    fn select_next_track(&mut self) {
        // TODO: check last
        let mut audio_tree = self.audio_tree.lock().unwrap();
        let max = audio_tree.audio_tracks.len();
        if audio_tree.selected_track_index + 1 < max.try_into().unwrap() {
            audio_tree.selected_track_index += 1;

            let mut tui = audio_tree.tui_state.lock().unwrap();
            *tui.selected_mut() = Some(audio_tree.selected_track_index.try_into().unwrap());
        }
    }

    fn select_prev_track(&mut self) {
        let mut audio_tree = self.audio_tree.lock().unwrap();
        if audio_tree.selected_track_index > 0 {
            audio_tree.selected_track_index -= 1;

            let mut tui = audio_tree.tui_state.lock().unwrap();
            *tui.selected_mut() = Some(audio_tree.selected_track_index.try_into().unwrap());
        }
    }
}
