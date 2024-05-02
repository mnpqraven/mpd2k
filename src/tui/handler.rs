use super::app::{AppState, NavigationRoute};
use crate::{client::PlaybackClient, error::AppError};
use crossterm::event::{KeyCode, KeyEvent};

/// Handles the key events and updates the state of [`AppState`].
pub fn handle_key_events<Client: PlaybackClient>(
    key_event: KeyEvent,
    app: &mut AppState<Client>,
) -> Result<(), AppError> {
    // page-dependent
    match app.navigation.current {
        NavigationRoute::Playback => match key_event.code {
            KeyCode::Char('l') => app.select_prev_track(),
            KeyCode::Char('n') => app.select_next_track(),
            // not final api, but there should be a char setter like this, and
            // a reader somewhere in the `AppState` impl
            KeyCode::Char('g') => {
                app.set_multi_key(KeyCode::Char('g'));
            }
            KeyCode::Char('G') => app.select_last_track(),
            KeyCode::Char('p') => app.pause_unpause(),
            KeyCode::Char('o') => app.play(),
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
                .library_client
                .try_lock()
                .is_ok_and(|client| !client.loading())
            {
                app.update_lib()
            }
        }

        KeyCode::Char('1') => app.navigate(NavigationRoute::Playback),
        KeyCode::Char('2') => app.navigate(NavigationRoute::Config),
        KeyCode::Char('3') => app.navigate(NavigationRoute::Help),
        KeyCode::Char('+') => app.volume_up(),
        KeyCode::Char('-') | KeyCode::Char('=') => app.volume_down(),
        _ => app.flush_multi_key(),
    }

    Ok(())
}
