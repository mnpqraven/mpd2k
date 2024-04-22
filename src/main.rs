pub mod error;
pub mod mpd;
pub mod tui;

use crate::mpd::MpdClient;
use clap::Parser;
use crossterm::{
    terminal::{self, ClearType},
    ExecutableCommand,
};
use error::AppError;
use mpd::CliClient;
use std::io::{self, Write};
use tui::draw_border;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt().init();

    let CliClient { addr, port } = CliClient::parse();
    let addr = format!("{addr}:{port}");

    let mut _client = MpdClient::init(addr).await?;

    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;

    draw_border(&stdout)?;

    // NOTE: OK
    // let _ = _client.command(mpd::MpdCommand::Play(200)).await;

    stdout.flush()?;
    Ok(())
}
