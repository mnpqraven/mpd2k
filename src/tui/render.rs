use super::{AudioTreeState, NavigationRoute, NavigationState};
use ratatui::{prelude::*, widgets::*};
use strum::IntoEnumIterator;

// TODO: file refactor
impl StatefulWidget for &AudioTreeState {
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
