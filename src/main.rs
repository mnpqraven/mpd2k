#![feature(if_let_guard)]
#![feature(let_chains)]

pub mod backend;
pub mod client;
pub mod dotfile;
pub mod error;
pub mod tui;

use backend::library::types::LibraryClient;
use client::{
    events::{AppToPlaybackEvent, PlaybackServer, PlaybackToAppEvent},
    PlaybackClient,
};
use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;
use tui::{
    app::{self, AppState},
    events::{Event, EventHandler},
    Tui,
};

// 60 fps
#[allow(non_snake_case)]
const TICK_RATE: u64 = 16;

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() -> Result<(), AppError> {
    // LOGGING
    let file_appender =
        tracing_appender::rolling::never(DotfileSchema::config_dir_path()?, "debug.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    // consume sender
    let (pb_tx, pb_rx) = mpsc::unbounded_channel::<AppToPlaybackEvent>();
    let (app_tx, app_rx) = mpsc::unbounded_channel::<PlaybackToAppEvent>();

    let playback_server = PlaybackServer::new_expr(&pb_tx, &app_tx);
    let playback_client = PlaybackClient::<LibraryClient>::new(&pb_tx, &app_tx);
    let mut app = AppState::new(pb_tx, app_tx);

    playback_server.spawn_listener(pb_rx);
    app.spawn_listener(app_rx, playback_client.arced());

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;

    let events = EventHandler::new(TICK_RATE);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    // MAIN EVENT LOOP
    while !app.exit {
        tui.draw(&app)?;

        match tui.events.next().await? {
            Event::Tick => app.tick()?,
            Event::Key(key_event) => app.handle_key_events(key_event)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // STDOUT CLEANUP
    app::teardown()?;

    Ok(())
}
