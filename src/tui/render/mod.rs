mod config;
mod help;
pub mod image;
mod playback;
pub mod tree;

use self::{
    config::ConfigContainer, help::HelpContainer, playback::PlaybackContainer,
    tree::LibraryTreeContainer,
};
use super::app::{AppState, NavigationRoute, NavigationState};
use crate::client::PlayableClient;
use ratatui::{prelude::*, widgets::*};
use std::time::{SystemTime, UNIX_EPOCH};
use strum::IntoEnumIterator;

// NOTE: is it better to refactor `render` mod to have file-based routing
impl<Client> Widget for &AppState<Client>
where
    Client: PlayableClient,
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
        let right_sidebar_width = self.tui_state.show_right_sidebar as u16 * 25;
        let split_ltr = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100 - right_sidebar_width),
                Constraint::Percentage(right_sidebar_width),
            ])
            .split(area);

        let split_ud = Layout::default()
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
        // TODO: safe unwrap
        let is_loading = self.client.get().unwrap().loading();

        // NAVBAR
        self.navigation.render(split_ud[0], buf);

        // MAIN BOX
        match self.navigation.current {
            NavigationRoute::Playback => PlaybackContainer(self, split_ud[1], buf),
            NavigationRoute::LibraryTree => LibraryTreeContainer(self, split_ud[1], buf),
            NavigationRoute::Config => ConfigContainer(self, split_ud[1], buf),
            NavigationRoute::Help => HelpContainer(self, split_ud[1], buf),
        };

        // RIGHT SIDEBAR
        if self.tui_state.show_right_sidebar {
            SidebarRight(clock, is_loading, split_ltr[1], buf);
        }
    }
}

#[allow(non_snake_case)]
fn SidebarRight(clock: u128, loading: bool, area: Rect, buf: &mut Buffer) {
    let loading_str = match loading {
        true => "loading...",
        false => "",
    };
    Paragraph::new(format!("{clock}\n{loading_str}"))
        .block(
            Block::new()
                .title("Now Playing [i]")
                .borders(Borders::all()),
        )
        .render(area, buf)
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
