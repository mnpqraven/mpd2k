use self::types::*;
use crate::{backend::library::cache::update_cache, dotfile::DotfileSchema};
use crossterm::event::{self, *};
use ratatui::prelude::*;
use std::io::{self};

mod event_handlers;
pub mod render;
pub mod types;

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
        let tree_arced = self.library_tree.clone();
        let loading_arced = self.loading_lib.clone();

        self.toggle_lib_loading();

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

    fn toggle_lib_loading(&mut self) {
        let mut loading = self.loading_lib.try_lock().unwrap();
        *loading = !*loading;
    }

    fn select_next_track(&mut self) {
        // TODO: check last
        let mut audio_tree = self.library_tree.lock().unwrap();
        let max = audio_tree.audio_tracks.len();
        if audio_tree.selected_track_index + 1 < max.try_into().unwrap() {
            audio_tree.selected_track_index += 1;

            let mut tui = audio_tree.tui_state.lock().unwrap();
            *tui.selected_mut() = Some(audio_tree.selected_track_index.try_into().unwrap());
        }
    }

    fn select_prev_track(&mut self) {
        let mut audio_tree = self.library_tree.lock().unwrap();
        if audio_tree.selected_track_index > 0 {
            audio_tree.selected_track_index -= 1;

            let mut tui = audio_tree.tui_state.lock().unwrap();
            *tui.selected_mut() = Some(audio_tree.selected_track_index.try_into().unwrap());
        }
    }
}
