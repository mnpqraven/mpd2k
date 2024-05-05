use super::app::{AppState, NavigationRoute};
use crate::{client::PlayableClient, error::AppError};
use crossterm::event::{KeyCode, KeyEvent};

/// Handles the key events and updates the state of [`AppState`].
pub fn handle_key_events<Client: PlayableClient>(
    key_event: KeyEvent,
    app: &mut AppState<Client>,
) -> Result<(), AppError> {
    match app.navigation.current {
        NavigationRoute::Playback => resolve_key_playback(app, &key_event)?,
        NavigationRoute::Config => resolve_key_config(app, &key_event)?,
        NavigationRoute::Help => resolve_key_help(app, &key_event)?,
    }

    resolve_key_universal(app, &key_event)?;

    Ok(())
}

fn resolve_key_universal<Client: PlayableClient>(
    app: &mut AppState<Client>,
    key_event: &KeyEvent,
) -> Result<(), AppError> {
    // universal
    if !app.tui_state.key_queue.is_empty() {
        return Ok(());
    }

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

        KeyCode::Char('1') => app.navigate(NavigationRoute::Playback),
        KeyCode::Char('2') => app.navigate(NavigationRoute::Config),
        KeyCode::Char('3') => app.navigate(NavigationRoute::Help),
        KeyCode::Char('i') => {
            app.tui_state.show_right_sidebar = !app.tui_state.show_right_sidebar;
        }
        KeyCode::Char('+') => app.client.get()?.volume_up(),
        KeyCode::Char('-') => app.client.get()?.volume_down(),
        _ => {}
    }

    Ok(())
}

fn resolve_key_playback<Client: PlayableClient>(
    app: &mut AppState<Client>,
    key_event: &KeyEvent,
) -> Result<(), AppError> {
    // multi keys get registered first

    match app.tui_state.key_queue.is_empty() {
        // NOTE: NON MULTI-KEY--------------------------------
        true => match key_event.code {
            KeyCode::Char('l') => app.client.get()?.select_prev_track(&mut app.tui_state)?,
            KeyCode::Char('n') => {
                app.client.get()?.select_next_track(&mut app.tui_state)?;
                // NOTE: update logic for image redraw
                // app.client.get()?.check_image(&mut app.tui_state)?;
            }
            KeyCode::Char('I') => {
                app.tui_state.show_left_sidebar = !app.tui_state.show_left_sidebar;
            }
            // NOTE: not final api, but there should be a char setter like
            // this, and a reader somewhere in the `AppState` impl
            KeyCode::Char('g') => app.set_multi_key(KeyCode::Char('g')),
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
        false => match key_event.code {
            KeyCode::Char('g') => {
                // gg
                if app.tui_state.key_queue.first() == Some(&KeyCode::Char('g')) {
                    app.flush_multi_key();
                    app.client.get()?.select_first_track(&mut app.tui_state)?;
                }
            }
            // !gg
            _ => app.flush_multi_key(),
        },
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
