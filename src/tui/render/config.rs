use crate::client::PlaybackClient;
use crate::tui::app::AppState;
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui::{
    prelude::*,
    widgets::{TableState, *},
};

#[allow(non_snake_case)]
pub fn ConfigContainer<Client>(app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlaybackClient,
    for<'a> &'a Client: StatefulWidget<State = TableState>,
{
    Block::new().borders(Borders::all()).render(area, buf);
}
