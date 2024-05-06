use super::playback::PlaybackBottom;
use crate::client::PlayableClient;
use crate::tui::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::{buffer::Buffer, layout::Rect};

#[allow(non_snake_case)]
pub fn LibraryTreeContainer<Client>(app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlayableClient,
{
    let rect_dir_seeker = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    if let Ok(client) = app.client.try_get() {
        let rows = client
            .albums()
            .iter()
            .map(|(meta, _tracks)| {
                Row::new([
                    Cell::from(meta.album_artist.clone().unwrap_or_default()),
                    Cell::from(meta.name.clone().unwrap_or_default()),
                    Cell::from(format!("{}", meta.date.0.unwrap())),
                ])
            })
            .collect::<Vec<Row>>();
        let widths = [
            Constraint::Ratio(2, 3),
            Constraint::Ratio(1, 3),
            Constraint::Min(10),
        ];

        // TODO: recheck scope
        let mut state = app.tui_state.library_table.lock().unwrap();
        ratatui::widgets::StatefulWidget::render(
            Table::new(rows, widths)
                .header(Row::new(["Artist", "Album", "Date"]))
                .column_spacing(1)
                .block(Block::new().borders(Borders::all())),
            rect_dir_seeker[0],
            buf,
            &mut state,
        );

        PlaybackBottom(&*client, rect_dir_seeker[1], buf);
    }
}
