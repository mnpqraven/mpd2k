#![feature(if_let_guard)]

pub mod backend;
pub mod client;
pub mod dotfile;
pub mod error;
pub mod tui;

use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
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

    let mut app = AppState::default();
    // WARN: DATA NEEDS TO BE INIT BEFORE THIS (stateful_tui)
    // STDOUT INIT
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    // 60 fps
    let events = EventHandler::new(16);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    while !app.exit {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // MAIN EVENT LOOP
    // let run = app.run(&mut terminal);

    // STDOUT CLEANUP
    app::teardown()?;

    Ok(())
}
