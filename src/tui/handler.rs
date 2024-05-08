use super::app::{AppState, KeyMode, NavigationRoute};
use crate::{
    client::{events::PlaybackToAppEvent, PlayableClient},
    error::AppError,
};
use crossterm::event::{KeyCode, KeyEvent};
use tracing::info;

impl<Client: PlayableClient> AppState<Client> {
    /// Handles the key events and updates the state of [`AppState`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match self.navigation.current {
            NavigationRoute::Playback => resolve_key_playback(self, &key_event)?,
            NavigationRoute::LibraryTree => resolve_key_library_tree(self, &key_event)?,
            NavigationRoute::Help => resolve_key_help(self, &key_event)?,
            NavigationRoute::Config => resolve_key_config(self, &key_event)?,
        }

        resolve_key_universal(self, &key_event)?;

        Ok(())
    }
}

fn resolve_key_universal<Client: PlayableClient>(
    app: &mut AppState<Client>,
    key_event: &KeyEvent,
) -> Result<(), AppError> {
    // universal
    if app.tui_state.key_mode == KeyMode::Normal {
        match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Char('u') => {
                // not loading
                if app.client.try_get().is_ok_and(|client| !client.loading()) {
                    app.client.update_lib(false)?;
                }
            }
            KeyCode::Char('U') => {
                // not loading
                if app.client.try_get().is_ok_and(|client| !client.loading()) {
                    app.client.update_lib(true)?;
                }
            }

            KeyCode::Char('r') => app.client.get()?.cycle_repeat(),
            KeyCode::Char('z') => app.client.get()?.toggle_shuffle(),

            KeyCode::Char('1') => app.navigate(NavigationRoute::Playback),
            KeyCode::Char('2') => app.navigate(NavigationRoute::LibraryTree),
            KeyCode::Char('3') => app.navigate(NavigationRoute::Config),
            KeyCode::Char('4') => app.navigate(NavigationRoute::Help),
            KeyCode::Char('i') => {
                app.tui_state.show_right_sidebar = !app.tui_state.show_right_sidebar;
            }
            KeyCode::Char('+') => app.client.get()?.volume_up(),
            KeyCode::Char('-') | KeyCode::Char('=') => app.client.get()?.volume_down(),
            _ => {}
        }
    }

    Ok(())
}

fn resolve_key_playback<Client: PlayableClient>(
    app: &mut AppState<Client>,
    key_event: &KeyEvent,
) -> Result<(), AppError> {
    // multi keys get registered first

    match &app.tui_state.key_mode {
        // NOTE: NON MULTI-KEY--------------------------------
        KeyMode::Normal => match key_event.code {
            KeyCode::Char('l') => app.client.get()?.select_prev_track(&mut app.tui_state)?,
            KeyCode::Char('n') => app.client.get()?.select_next_track(&mut app.tui_state)?,
            KeyCode::Char('I') => {
                app.tui_state.show_left_sidebar = !app.tui_state.show_left_sidebar;
            }
            // NOTE: not final api, but there should be a char setter like
            // this, and a reader somewhere in the `AppState` impl
            KeyCode::Char('g') => {
                app.set_keymode(KeyMode::Multi(vec![KeyCode::Char('g')]));
            }
            KeyCode::Char('G') => app.client.get()?.select_last_track(&mut app.tui_state)?,
            KeyCode::Char('p') => app.client.get()?.pause_unpause(),
            KeyCode::Char('o') => {
                if let Ok(tui) = app.tui_state.playback_table.lock() {
                    app.client.get()?.play(&tui)?;
                }
            }
            _ => {}
        },
        // NOTE MULTI-KEY--------------------------------
        KeyMode::Multi(key_queue) => match key_event.code {
            KeyCode::Char('g') => {
                // gg
                if key_queue.first() == Some(&KeyCode::Char('g')) {
                    app.flush_multi_key();
                    app.set_keymode(KeyMode::Normal);
                    app.client.get()?.select_first_track(&mut app.tui_state)?;
                }
            }
            // !gg
            _ => {
                app.set_keymode(KeyMode::Normal);
                app.flush_multi_key();
            }
        },
        KeyMode::Editing => {}
    }

    Ok(())
}

fn resolve_key_config<Client: PlayableClient>(
    _app: &mut AppState<Client>,
    _key_event: &KeyEvent,
) -> Result<(), AppError> {
    Ok(())
}

fn resolve_key_help<Client: PlayableClient>(
    _app: &mut AppState<Client>,
    _key_event: &KeyEvent,
) -> Result<(), AppError> {
    Ok(())
}

fn resolve_key_library_tree<Client: PlayableClient>(
    _app: &mut AppState<Client>,
    _key_event: &KeyEvent,
) -> Result<(), AppError> {
    Ok(())
}
