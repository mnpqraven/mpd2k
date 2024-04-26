use super::types::{AppState, NavigationRoute, NavigationState};
use crate::{backend::library::AudioTrack, client::library::LibraryClient};
use ratatui::{prelude::*, widgets::*};
use std::time::{SystemTime, UNIX_EPOCH};
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
    /// // |------------------------------|
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
                // 2 for borders + name + prolly shortcut
                Constraint::Length(3),
                // fill rest
                Constraint::Min(10),
            ])
            .split(container_ltr[0]);

        let clock = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let sidebar_right = Paragraph::new(format!("sidebar right\n{clock}"))
            .block(Block::new().borders(Borders::all()));

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
            Ok(audio_tree) if audio_tree.tui_state.try_lock().is_ok() => {
                let tui_state = audio_tree.tui_state.lock().unwrap();
                let get = tui_state
                    .selected()
                    .and_then(|e| audio_tree.audio_tracks.get(e));
                drop(tui_state);
                let mainbox_left_component = main_left(get);
                mainbox_left_component.render(mainbox_left, buf);

                let mut tui_state = audio_tree.tui_state.try_lock().unwrap();
                audio_tree.render(mainbox_right, buf, &mut tui_state)
            }
            _ => {
                let mut empty = TableState::default();
                LibraryClient::default().render(mainbox_right, buf, &mut empty);
            }
        };
        // frame.render_widget(main, container_ltr[0]);
        // RIGHT SIDEBAR
        sidebar_right.render(container_ltr[1], buf);
    }
}

fn main_left<'a>(data: Option<&AudioTrack>) -> Paragraph<'a> {
    let block = Block::new().borders(Borders::all());
    match data {
        None => Paragraph::default().block(block),
        Some(data) => {
            let text = vec![
                Line::from(format!("Album: {}", data.album.clone().unwrap_or_default())),
                Line::from(format!(
                    "Album Artist: {}",
                    data.album_artist.clone().unwrap_or_default()
                )),
                Line::from("Year: 2024"),
            ];
            Paragraph::new(text).block(block)
        }
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
