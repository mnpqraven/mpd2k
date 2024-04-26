use super::types::*;
use crate::{client::PlaybackEvent, error::AppError};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

/// Handles the key events and updates the state of [`AppState`].
pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut AppState,
    playback_tx: UnboundedSender<PlaybackEvent>,
) -> Result<(), AppError> {
    // universal
    match key_event.code {
        KeyCode::Char('q') => app.exit(),
        KeyCode::Char('u') => {
            // not loading
            if app
                .library_loading
                .try_lock()
                .is_ok_and(|loading| !*loading)
            {
                app.update_lib()
            }
        }

        KeyCode::Char('1') => app.navigate(NavigationRoute::Playback),
        KeyCode::Char('2') => app.navigate(NavigationRoute::Config),
        _ => {}
    }

    // page-dependent
    match app.navigation.current {
        NavigationRoute::Playback => match key_event.code {
            KeyCode::Char('n') => app.select_next_track(),
            KeyCode::Char('p') => app.select_prev_track(),
            KeyCode::Char('o') => {
                app.play();
                let play = playback_tx.send(PlaybackEvent::Play);
                info!("sending play command {:?}", play);
            }
            _ => {}
        },
        NavigationRoute::Config => {}
    }
    Ok(())
}
