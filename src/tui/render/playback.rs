use crate::backend::library::types::AudioTrack;
use crate::client::library::CurrentTrack;
use crate::client::PlayableClient;
use crate::tui::app::AppState;
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui::{
    prelude::*,
    widgets::{TableState, *},
};
use std::time::Duration;

#[allow(non_snake_case)]
pub fn PlaybackContainer<Client>(app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlayableClient,
{
    let rect_dir_seeker = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let left_sidebar_width = app.tui_state.show_left_sidebar as u16 * 40;
    let mainbox_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_sidebar_width),
            Constraint::Percentage(100 - left_sidebar_width),
        ])
        .split(rect_dir_seeker[0]);
    let mainbox_left = mainbox_layout[0];
    let mainbox_right = mainbox_layout[1];

    if let Ok(client) = app.client.try_get()
        && let Ok(mut tui_state) = app.tui_state.playback_table.try_lock()
    {
        MainBoxLeft(&*client, mainbox_left, buf, &mut tui_state);

        MainboxRight(&*client, mainbox_right, buf, &mut tui_state);

        PlaybackBottom(&*client, rect_dir_seeker[1], buf);
    };
}

#[allow(non_snake_case)]
fn MainBoxLeft<Client: PlayableClient>(
    data: &Client,
    area: Rect,
    buf: &mut Buffer,
    state: &mut TableState,
) {
    let tracks = data.audio_tracks();
    let current_track = state.selected().and_then(|f| tracks.get(f));

    let block = Block::new()
        .title("Selected Track [I]")
        .borders(Borders::all());

    let paragraph = match current_track {
        None => Paragraph::default().block(block),
        Some(data) => {
            let mut text = Vec::new();
            if let Some(album) = &data.album {
                text.push(Line::from(format!("Album: {}", album)))
            }
            if let Some(album_artist) = &data.album_artist {
                text.push(Line::from(format!("Album Artist: {}", album_artist)))
            }
            if let Some(date) = &data.date.0 {
                text.push(Line::from(format!("Released: {}", date)))
            }

            Paragraph::new(text).wrap(Wrap { trim: true }).block(block)
        }
    };
    paragraph.render(area, buf)
}

#[allow(non_snake_case)]
fn MainboxRight<Client: PlayableClient>(
    data: &Client,
    area: Rect,
    buf: &mut Buffer,
    state: &mut TableState,
) {
    let rows = data
        .audio_tracks()
        .iter()
        .map(|data| {
            Row::new([
                Cell::from(data.name.clone()),
                Cell::from(data.artist.to_owned().unwrap_or_default()),
                Cell::from(data.track_no.unwrap_or_default().to_string()),
            ])
        })
        .collect::<Vec<Row>>();

    let widths = [
        Constraint::Ratio(2, 3),
        Constraint::Ratio(1, 3),
        Constraint::Min(3),
    ];

    ratatui::widgets::StatefulWidget::render(
        Table::new(rows, widths)
            .header(
                Row::new([Cell::from("Title"), Cell::from("Artist"), Cell::from("#")])
                    .bottom_margin(1),
            )
            .column_spacing(1)
            .block(Block::new().borders(Borders::all()))
            .highlight_style(Style::new().black().on_white()),
        area,
        buf,
        state,
    )
}

#[allow(non_snake_case)]
pub fn PlaybackBottom<Client: PlayableClient>(client: &Client, area: Rect, buf: &mut Buffer) {
    let current_track = client.current_track();

    let block = Block::new().borders(Borders::all());
    let layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Min(10),
            Constraint::Length(17),
            Constraint::Length(8),
            Constraint::Max(20),
        ],
    )
    .margin(1)
    .split(area);
    block.render(area, buf);

    let _symbol_elapsed = Span::raw(">");
    let symbol_empty = Span::raw("-");
    let playback_line_width = layout[0].width;
    // TODO: math out rendering logic for elapsed duration
    let playback_line = match current_track {
        Some(_) => {
            Vec::from_iter(std::iter::repeat(symbol_empty).take(playback_line_width as usize))
        }
        None => Vec::from_iter(std::iter::repeat(symbol_empty).take(playback_line_width as usize)),
    };

    let line = Line::from(playback_line);

    let duration = timestamp(&current_track.map(|e| e.duration));
    let duration = Line::from(format!("0:00 / {}", duration)).alignment(Alignment::Right);
    let volume = client.volume_percentage();
    let volume = Line::from(format!("{} %", volume)).alignment(Alignment::Right);
    let status = Line::from(vec![" Rep ".into(), " Loop ".into(), " Upd ".into()])
        .alignment(Alignment::Right);
    Paragraph::new(line).render(layout[0], buf);
    Paragraph::new(duration)
        .block(
            Block::new()
                .borders(Borders::LEFT | Borders::RIGHT)
                .padding(Padding::horizontal(1)),
        )
        .render(layout[1], buf);
    Paragraph::new(volume)
        .block(
            Block::new()
                .borders(Borders::RIGHT)
                .padding(Padding::horizontal(1)),
        )
        .render(layout[2], buf);
    Paragraph::new(status).render(layout[3], buf);
}

fn timestamp(dur: &Option<Duration>) -> String {
    match dur {
        Some(dur) => {
            let ms = dur.as_secs();
            let mins = ms / 60;
            let ms_rest = ms - mins * 60;
            format!("{mins}:{ms_rest}")
        }
        None => "00:00".into(),
    }
}
