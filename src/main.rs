#![feature(if_let_guard)]

pub mod backend;
pub mod client;
pub mod dotfile;
pub mod error;
pub mod tui;

use core::panic;

use client::{handle_playback_event, PlaybackClient, PlaybackEvent};
use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{runtime::Handle, sync::oneshot};
use tracing::info;
use tui::{
    app::{self},
    events::{Event, EventHandler},
    handler::handle_key_events,
    types::AppState,
    Tui,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // LOGGING
    let file_appender =
        tracing_appender::rolling::never(DotfileSchema::config_dir_path()?, "debug.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    let mut app = AppState::new();
    // WARN: DATA NEEDS TO BE INIT BEFORE THIS (stateful_tui)
    // STDOUT INIT
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    // 60 fps
    #[allow(non_snake_case)]
    let TICK_RATE = 16;
    let events = EventHandler::new(TICK_RATE);
    let mut playback_events = PlaybackClient::new();
    let playback_tx = playback_events.sender.clone();
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    // MAIN EVENT LOOP
    while !app.exit {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Tick => {
                app.tick();
                let _ = playback_tx.send(PlaybackEvent::Tick);
            }
            Event::Key(key_event) => handle_key_events(key_event, &mut app, playback_tx.clone())?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }

        playback_events.handle(&mut app).await?;
    }

    // STDOUT CLEANUP
    app::teardown()?;

    Ok(())
}
