use crate::client::PlayableClient;
use crate::tui::app::AppState;
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui::{
    prelude::*,
    widgets::{TableState, *},
};

#[allow(non_snake_case)]
pub fn HelpContainer<Client>(app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlayableClient,
    for<'a> &'a Client: StatefulWidget<State = TableState>,
{
    Block::new().borders(Borders::all()).render(area, buf);
}
