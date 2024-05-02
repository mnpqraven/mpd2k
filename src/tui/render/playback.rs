use std::time::Duration;

use crate::backend::library::AudioTrack;
use crate::client::PlaybackClient;
use crate::tui::app::AppState;
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui::{
    prelude::*,
    widgets::{TableState, *},
};

#[allow(non_snake_case)]
pub fn PlaybackContainer<Client>(app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlaybackClient,
    for<'a> &'a Client: StatefulWidget<State = TableState>,
{
    let rect_dir_seeker = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(10), Constraint::Length(3)])
        .split(area);
    let mainbox_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rect_dir_seeker[0]);
    let mainbox_left = mainbox_layout[0];
    let mainbox_right = mainbox_layout[1];

    if let Ok(audio_tree) = app.library_client.try_lock()
        && let Ok(mut tui_state) = app.tui_state.try_lock()
    {
        let current_track = tui_state
            .selected()
            .and_then(|f| audio_tree.audio_tracks().get(f));

        audio_tree.render(mainbox_right, buf, &mut tui_state);

        MainBoxLeft(current_track, mainbox_left, buf);

        PlaybackBottom(
            current_track,
            audio_tree.current_track().map(|e| e.duration),
            audio_tree.volume_percentage(),
            rect_dir_seeker[1],
            buf,
        );
    };
}

#[allow(non_snake_case)]
fn MainBoxLeft(data: Option<&AudioTrack>, area: Rect, buf: &mut Buffer) {
    let block = Block::new().borders(Borders::all());
    let paragraph = match data {
        None => Paragraph::default().block(block),
        Some(data) => {
            let mut text = Vec::new();
            if let Some(album) = &data.album {
                text.push(Line::from(format!("Album: {}", album)))
            }
            if let Some(album_artist) = &data.album_artist {
                text.push(Line::from(format!("Album Artist: {}", album_artist)))
            }
            if let Some(date) = &data.date {
                text.push(Line::from(format!("Released: {}", date)))
            }

            Paragraph::new(text).block(block)
        }
    };
    paragraph.render(area, buf)
}

#[allow(non_snake_case)]
fn PlaybackBottom(
    data: Option<&AudioTrack>,
    dur: Option<Duration>,
    volume_percentage: u8,
    area: Rect,
    buf: &mut Buffer,
) {
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
    let playback_line = match data {
        Some(_) => {
            Vec::from_iter(std::iter::repeat(symbol_empty).take(playback_line_width as usize))
        }
        None => Vec::from_iter(std::iter::repeat(symbol_empty).take(playback_line_width as usize)),
    };

    let line = Line::from(playback_line);

    let duration = Line::from(format!("0:00 / {}", timestamp(&dur))).alignment(Alignment::Right);
    let volume = Line::from(format!("{} %", volume_percentage)).alignment(Alignment::Right);
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