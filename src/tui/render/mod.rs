mod config;
mod help;
mod playback;

use self::{config::ConfigContainer, help::HelpContainer, playback::PlaybackContainer};
use super::app::{AppState, NavigationRoute, NavigationState};
use crate::client::{library::LibraryClient, PlayableClient};
use ratatui::{
    prelude::*,
    widgets::{TableState, *},
};
use std::time::{SystemTime, UNIX_EPOCH};
use strum::IntoEnumIterator;

// NOTE: is it better to refactor `render` mod to have file-based routing
impl<Client> Widget for &AppState<Client>
where
    Client: PlayableClient,
    for<'a> &'a Client: StatefulWidget<State = TableState>,
{
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
        let split_ltr = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(area);

        let split_navbar_mainbox = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                // top navigation bar
                Constraint::Length(3),
                // fill rest
                Constraint::Min(10),
            ])
            .split(split_ltr[0]);

        let clock = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let is_loading = self
            .client
            .inner
            .lock()
            .map(|e| e.loading())
            .unwrap();

        // NAVBAR
        self.navigation.render(split_navbar_mainbox[0], buf);

        // MAIN BOX
        match self.navigation.current {
            NavigationRoute::Playback => PlaybackContainer(self, split_navbar_mainbox[1], buf),
            NavigationRoute::Config => ConfigContainer(self, split_navbar_mainbox[1], buf),
            NavigationRoute::Help => HelpContainer(self, split_navbar_mainbox[1], buf),
        };

        // RIGHT SIDEBAR
        SidebarRight(clock, is_loading, split_ltr[1], buf);
    }
}

#[allow(non_snake_case)]
fn SidebarRight(clock: u128, loading: bool, area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!("{clock} {loading}"))
        .block(Block::new().borders(Borders::all()))
        .render(area, buf)
}

// TODO: file refactor
// TODO: render for mpd client too
impl StatefulWidget for &LibraryClient {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let rows = self
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
                true => Paragraph::new(Line::from(route.to_string()).alignment(Alignment::Center))
                    .black()
                    .on_white(),
                false => Paragraph::new(Line::from(route.to_string()).alignment(Alignment::Center)),
            })
            .enumerate()
            .for_each(|(index, para)| para.render(nav_item_layouts[index], buf))
    }
}
