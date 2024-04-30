use super::types::*;
use crate::{client::PlaybackClient, error::AppError};
use crossterm::event::{KeyCode, KeyEvent};

/// Handles the key events and updates the state of [`AppState`].
pub fn handle_key_events<Client: PlaybackClient>(
    key_event: KeyEvent,
    app: &mut AppState<Client>,
) -> Result<(), AppError> {
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
        KeyCode::Char('+') => app.volume_up(),
        KeyCode::Char('-') | KeyCode::Char('=') => app.volume_down(),
        _ => {}
    }

    // page-dependent
    match app.navigation.current {
        NavigationRoute::Playback => match key_event.code {
            KeyCode::Char('l') => app.select_prev_track(),
            KeyCode::Char('n') => app.select_next_track(),
            KeyCode::Char('p') => app.pause_unpause(),
            KeyCode::Char('o') => app.play(),
            _ => {}
        },
        NavigationRoute::Config => {}
    }
    Ok(())
}
