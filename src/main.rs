#![feature(if_let_guard)]
#![feature(let_chains)]

pub mod backend;
pub mod client;
pub mod constants;
pub mod dotfile;
pub mod error;
pub mod tui;

use backend::library::types::LibraryClient;
use client::events::{AppToPlaybackEvent, PlaybackServer, PlaybackToAppEvent};
use constants::{HASH_CONCURRENT_LIMIT, TICK_RATE};
use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{
    runtime::{Builder, Handle},
    sync::mpsc,
};
use tui::{
    app::{self, AppState},
    events::{Event, EventHandler},
    Tui,
};

fn main() -> Result<(), AppError> {
    // NOTE: clap arg here if needed

    let hashing_rt = Builder::new_multi_thread()
        .thread_name("hashing thread")
        .worker_threads(HASH_CONCURRENT_LIMIT)
        .enable_all()
        .build()
        .unwrap();
    let hashing_handle = hashing_rt.handle();

    let main_rt = Builder::new_multi_thread()
        .thread_name("main thread")
        .enable_all()
        .build()
        .unwrap();

    main_rt.block_on(_main(hashing_handle))?;

    Ok(())
}

async fn _main(hash_handle: &Handle) -> Result<(), AppError> {
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
    // TODO: refactor Mutex to RwLock
    let mut app = AppState::<LibraryClient>::new(pb_tx, app_tx, hash_handle.clone());

    playback_server.spawn_listener(pb_rx);
    app.spawn_listener(app_rx);

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
