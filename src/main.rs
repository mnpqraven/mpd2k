pub mod backend;
pub mod client;
pub mod dotfile;
pub mod error;
pub mod tui;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use dotfile::DotfileSchema;
use error::AppError;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui::StatefulTui;

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

    // STDOUT INIT
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    // STATES
    let _dotfile = DotfileSchema::parse()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // MAIN EVENT LOOP
    let stateful_tui = StatefulTui::default()
        // experimental flag
        .load_all()?
        .run(&mut terminal);

    // STDOUT CLEANUP
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(stateful_tui?)
}
