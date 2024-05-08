#![feature(if_let_guard)]
#![feature(let_chains)]

pub mod backend;
pub mod client;
pub mod dotfile;
pub mod error;
pub mod tui;

use backend::library::types::LibraryClient;
use client::{
    events::{AppToPlaybackEvent, PlaybackServer},
    PlayableClient,
};
use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{runtime::Builder, sync::mpsc};
use tui::{
    app::{self, AppState},
    events::{Event, EventHandler},
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

    // PLAYBACK SERVER setup
    let playback_rt = Builder::new_current_thread().build().unwrap();
    let playback_handle = playback_rt.handle().to_owned();

    // let mut app = AppState::<LibraryClient>::new(pb_sender.clone(), app_listener);

    // consume sender
    let playback_server = PlaybackServer::new_expr(playback_handle);
    let (client, app_send) = LibraryClient::new();
    let mut app = AppState::from_client(client, playback_server.sender.clone(), app_send);

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;

    // 60 fps
    #[allow(non_snake_case)]
    let TICK_RATE = 16;
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
    app.client.teardown()?;
    playback_rt.shutdown_background();
    app::teardown()?;

    Ok(())
}
