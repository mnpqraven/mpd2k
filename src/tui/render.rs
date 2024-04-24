use super::{AudioTreeState, NavigationRoute, NavigationState};
use ratatui::{prelude::*, widgets::*};
use strum::IntoEnumIterator;

// TODO: file refactor
impl Widget for &AudioTreeState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let current = self.selected_track_index;
        let constraints =
            Vec::from_iter(std::iter::repeat(Constraint::Max(1)).take(area.height.into()));

        let container_for_border = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);
        Block::new()
            .borders(Borders::all())
            .padding(Padding::uniform(1))
            .render(container_for_border[0], buf);

        let song_item_layouts = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(container_for_border[0]);

        // TODO: impl scrolling

        self.audio_tracks
            .iter()
            .take(area.height.into())
            .enumerate()
            .map(|(index, data)| match index == current.try_into().unwrap() {
                true => Paragraph::new(Line::from(data.name.clone()))
                    .block(Block::new().padding(Padding::horizontal(1)))
                    .black()
                    .on_white(),
                false => Paragraph::new(Line::from(data.name.clone()))
                    .block(Block::new())
                    .white()
                    .on_black(),
            })
            .enumerate()
            .for_each(|(index, para)| para.render(song_item_layouts[index], buf));
    }
}

impl Widget for &NavigationState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let constraints = Vec::from_iter(
            std::iter::repeat(Constraint::Min(1)).take(NavigationRoute::iter().len()),
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
                true => Line::from(route.to_string()).centered().black().on_white(),
                false => Line::from(route.to_string()).centered().white().on_black(),
            })
            .enumerate()
            .for_each(|(index, line)| Paragraph::new(line).render(nav_item_layouts[index], buf))
    }
}
