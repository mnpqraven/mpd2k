use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, QueueableCommand,
};
use std::io::{self, Stdout};

pub fn draw_border(mut stdout: &Stdout) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    for y in 0..height {
        for x in 0..width {
            if (y == 0 || y == height - 1) || (x == 0 || x == width - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("─".dark_grey()))?;
            }
        }
    }
    Ok(())
}
