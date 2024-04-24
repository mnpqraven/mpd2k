use super::{AudioTreeState, NavigationRoute, NavigationState};
use ratatui::{prelude::*, widgets::*};
use strum::IntoEnumIterator;

// TODO: file refactor
impl StatefulWidget for &AudioTreeState {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = self
            .audio_tracks
            .iter()
            .map(|data| Text::from(data.name.to_owned()))
            .collect::<Vec<Text>>();

        ratatui::widgets::StatefulWidget::render(
            List::new(items)
                .highlight_style(Style::new().black().on_white())
                .scroll_padding(3),
            area,
            buf,
            state,
        )
    }
}

// impl Widget for &AudioTreeState {
//     fn render(self, area: Rect, buf: &mut Buffer)
//     where
//         Self: Sized,
//     {
//         let current = self.selected_track_index;
//         let container_for_border = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints(vec![Constraint::Percentage(100)])
//             .split(area);
//         Block::new()
//             .borders(Borders::all())
//             .padding(Padding::uniform(1))
//             .render(container_for_border[0], buf);
//
//         let item_constraints =
//             Vec::from_iter(std::iter::repeat(Constraint::Max(1)).take(area.height.into()));
//         let song_item_layouts = Layout::default()
//             .direction(Direction::Vertical)
//             .margin(1)
//             .constraints(item_constraints)
//             .split(container_for_border[0]);
//
//         // TODO: impl scrolling
//
//         // self.audio_tracks
//         //     .iter()
//         //     .take(area.height.into())
//         //     .enumerate()
//         //     .for_each(|(index, data)| {
//         //         let flex_div = Layout::new(
//         //             Direction::Horizontal,
//         //             vec![
//         //                 Constraint::Fill(2),
//         //                 Constraint::Fill(2),
//         //                 Constraint::Max(5),
//         //                 Constraint::Fill(1),
//         //             ],
//         //         )
//         //         .split(song_item_layouts[index]);
//         //         let is_selected = current == index.try_into().unwrap();
//         //         let (fg, bg) = match is_selected {
//         //             true => (Color::Black, Color::White),
//         //             false => (Color::White, Color::Reset),
//         //         };
//         //
//         //         if let Some(album_artist) = &data.album_artist {
//         //             Paragraph::new(album_artist.to_owned())
//         //                 .fg(fg)
//         //                 .bg(bg)
//         //                 .render(flex_div[0], buf);
//         //         }
//         //
//         //         // name
//         //         Paragraph::new(data.name.to_string())
//         //             .fg(fg)
//         //             .bg(bg)
//         //             .render(flex_div[1], buf);
//         //         // track no
//         //         if let Some(track_no) = data.track_no {
//         //             Paragraph::new(format!("{:0>2}", track_no))
//         //                 .fg(fg)
//         //                 .bg(bg)
//         //                 .render(flex_div[2], buf);
//         //         }
//         //         // rest
//         //         Paragraph::new(String::new())
//         //             .fg(fg)
//         //             .bg(bg)
//         //             .render(flex_div[2], buf);
//         //     });
//
//         let texts = self
//             .audio_tracks
//             .iter()
//             .take(area.height.into())
//             .map(|data| Text::from(data.name.to_owned()))
//             .collect::<Vec<Text>>();
//
//         ratatui::widgets::Widget::render(
//             List::new(texts)
//                 .direction(ListDirection::TopToBottom)
//                 .highlight_style(Style::new().black().on_white())
//                 .scroll_padding(3),
//             container_for_border[0],
//             buf,
//         );
//     }
// }

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
