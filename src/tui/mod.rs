use self::types::*;
use crate::client::library::LibraryClient;
use crate::client::Playback;
use crate::{backend::library::cache::update_cache, dotfile::DotfileSchema};
use crossterm::event::{self, *};
use ratatui::prelude::*;
use std::io::{self};

mod event_handlers;
pub mod render;
pub mod types;

impl AppState {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
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
            KeyCode::Char('u') => {
                // not loading
                if self
                    .library_loading
                    .try_lock()
                    .is_ok_and(|loading| !*loading)
                {
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
                KeyCode::Char('o') => self.play(),
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
        let tree_arced = self.library_client.clone();
        let loading_arced = self.library_loading.clone();

        self.toggle_loading();

        tokio::spawn(async move {
            let cfg = DotfileSchema::parse().unwrap();
            // let mut audio_tree = tree_arced.lock().unwrap();
            // TODO: caching
            // audio_tree.audio_tracks = load_all_tracks(&cfg).unwrap();

            // NOTE: caching impl
            let _ = update_cache(&cfg, tree_arced);
            // if let Ok(tracks) = update_cache(&cfg, tree_arced) {
            //     audio_tree.audio_tracks = tracks;
            // };

            let mut loading = loading_arced.lock().unwrap();
            *loading = !*loading;
        });
    }

    fn navigate(&mut self, to: NavigationRoute) {
        self.navigation.current = to
    }

    fn toggle_loading(&mut self) {
        let mut loading = self.library_loading.try_lock().unwrap();
        *loading = !*loading;
    }

    fn play(&mut self) {
        let lib_arc = self.library_client.clone();
        tokio::spawn(async move {
            let lib = lib_arc.lock().unwrap();
            let tui_state = lib.tui_state.lock().unwrap();

            let index = tui_state.selected().unwrap();
            let track = lib.audio_tracks.get(index).unwrap();

            drop(tui_state);
            // FIX: this causes blocking
            // till the song is over
            lib.play(Some(track)).unwrap();
        });
    }

    fn select_next_track(&mut self) {
        // TODO: check last
        let audio_tree = self.library_client.lock().unwrap();
        let max = audio_tree.audio_tracks.len();
        let mut tui_state = audio_tree.tui_state.lock().unwrap();
        match tui_state.selected() {
            Some(index) => {
                if index + 1 < max {
                    tui_state.select(Some(index + 1));
                }
            }
            None => tui_state.select(Some(0)),
        }
    }

    fn select_prev_track(&mut self) {
        let audio_tree = self.library_client.lock().unwrap();
        let mut tui_state = audio_tree.tui_state.lock().unwrap();
        match tui_state.selected() {
            Some(index) => {
                if index > 1 {
                    tui_state.select(Some(index - 1))
                }
            }
            None => tui_state.select(Some(0)),
        }
    }
}
