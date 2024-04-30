use super::types::{AppState, NavigationRoute, NavigationState};
use crate::{backend::library::AudioTrack, client::library::LibraryClient};
use ratatui::{prelude::*, widgets::*};
use std::{
    sync::MutexGuard,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use strum::IntoEnumIterator;

impl Widget for &AppState {
    /// TODO:
    /// ```
    /// // |-------------------------------|
    /// // |        navbar        |        |
    /// // |----------------------|        |
    /// // |      |               |  track |
    /// // | info |     maindex   |  info  |
    /// // |      |               |        |
    /// // |-------------------------------|
    /// // | seek bar | dur | opt |        |
    /// // |-------------------------------|
    /// ```
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let container_ltr = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(area);
        let container_left_ud = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                // top navigation bar
                Constraint::Length(3),
                // fill rest
                Constraint::Min(10),
                // bottom playback bar
                Constraint::Length(3),
            ])
            .split(container_ltr[0]);

        let clock = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let is_loading = self.library_client.lock().map(|e| e.loading).unwrap();

        // NAVBAR
        self.navigation.render(container_left_ud[0], buf);
        // frame.render_widget(navbar, container_left_ud[0]);
        // MAIN BOX
        let mainbox_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(container_left_ud[1]);
        let mainbox_left = mainbox_layout[0];
        let mainbox_right = mainbox_layout[1];

        // mainbox_left_component.render(area, buf)

        match self.library_client.try_lock() {
            Ok(audio_tree) if self.tui_state.try_lock().is_ok() => {
                let lookup_fn = |e: MutexGuard<TableState>| {
                    e.selected().and_then(|f| audio_tree.audio_tracks.get(f))
                };
                let track = self.tui_state.lock().map(lookup_fn).unwrap();

                let mut tui_state = self.tui_state.try_lock().unwrap();
                audio_tree.render(mainbox_right, buf, &mut tui_state);

                MainBoxLeft(track, mainbox_left, buf);

                PlaybackBottom(
                    track,
                    audio_tree.current_track.clone().map(|e| e.duration),
                    audio_tree.volume_percentage(),
                    container_left_ud[2],
                    buf,
                );
            }
            _ => {
                let mut empty = TableState::default();
                // TODO: default render
                // LibraryClient::default().render(mainbox_right, buf, &mut empty);
            }
        };

        // RIGHT SIDEBAR
        SidebarRight(clock, is_loading, container_ltr[1], buf);
    }
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

#[allow(non_snake_case)]
fn SidebarRight(clock: u128, loading: bool, area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!("{clock} {loading}"))
        .block(Block::new().borders(Borders::all()))
        .render(area, buf)
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

// TODO: file refactor
impl StatefulWidget for &LibraryClient {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let rows = self
            .audio_tracks
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
}

impl Widget for &NavigationState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let item_count = NavigationRoute::iter().len();
        let constraints = Vec::from_iter(
            std::iter::repeat(Constraint::Ratio(1, item_count.try_into().unwrap()))
                .take(item_count),
        );

        let container_div = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);
        Block::new()
            .borders(Borders::all())
            .padding(Padding::uniform(1))
            .render(container_div[0], buf);

        let nav_item_layouts = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(constraints)
            .split(container_div[0]);

        NavigationRoute::iter()
            .map(|route| match self.current == route {
                true => Paragraph::new(
                    Line::from(
                        Span::from(route.to_string()), // .style(Style::new().fg(Color::Black).bg(Color::White)),
                    )
                    .alignment(Alignment::Center),
                )
                .black()
                .on_white(),
                false => Paragraph::new(Line::from(route.to_string()).alignment(Alignment::Center)),
            })
            .enumerate()
            .for_each(|(index, para)| para.render(nav_item_layouts[index], buf))
    }
}
