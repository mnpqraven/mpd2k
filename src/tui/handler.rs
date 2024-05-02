use super::app::{AppState, NavigationRoute};
use crate::{client::PlayableClient, error::AppError};
use crossterm::event::{KeyCode, KeyEvent};

/// Handles the key events and updates the state of [`AppState`].
pub fn handle_key_events<Client: PlayableClient>(
    key_event: KeyEvent,
    app: &mut AppState<Client>,
) -> Result<(), AppError> {
    // page-dependent
    match app.navigation.current {
        NavigationRoute::Playback => match key_event.code {
            KeyCode::Char('l') => {
                if let Ok(mut tui) = app.tui_state.lock() {
                    app.client.get()?.select_prev_track(&mut tui)
                }
            }
            KeyCode::Char('n') => {
                if let Ok(mut tui) = app.tui_state.lock() {
                    app.client.get()?.select_next_track(&mut tui)
                }
            }
            // NOTE: not final api, but there should be a char setter like
            // this, and a reader somewhere in the `AppState` impl
            KeyCode::Char('g') => {
                app.set_multi_key(KeyCode::Char('g'));
            }
            KeyCode::Char('G') => {
                if let Ok(mut tui) = app.tui_state.lock() {
                    app.client.get()?.select_last_track(&mut tui);
                }
            }
            KeyCode::Char('p') => app.client.get()?.pause_unpause(),
            KeyCode::Char('o') => {
                if let Ok(tui) = app.tui_state.lock() {
                    app.client.get()?.play(&tui)?;
                }
            }
            _ => {}
        },
        NavigationRoute::Config => {}
        NavigationRoute::Help => {}
    }

    // universal
    match key_event.code {
        KeyCode::Char('q') => app.exit(),
        KeyCode::Char('u') => {
            // not loading
            if app
                .client
                .inner
                .try_lock()
                .is_ok_and(|client| !client.loading())
            {
                app.client.update_lib()?;
            }
        }

        KeyCode::Char('1') => app.navigate(NavigationRoute::Playback),
        KeyCode::Char('2') => app.navigate(NavigationRoute::Config),
        KeyCode::Char('3') => app.navigate(NavigationRoute::Help),
        KeyCode::Char('+') => app.client.get()?.volume_up(),
        KeyCode::Char('-') => app.client.get()?.volume_down(),
        _ => app.flush_multi_key(),
    }

    Ok(())
}
