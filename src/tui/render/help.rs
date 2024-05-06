use crate::client::PlayableClient;
use crate::tui::app::AppState;
use ratatui::widgets::{TableState, *};
use ratatui::{buffer::Buffer, layout::Rect};

#[allow(non_snake_case)]
pub fn HelpContainer<Client>(_app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlayableClient,
    for<'a> &'a Client: StatefulWidget<State = TableState>,
{
    Block::new().borders(Borders::all()).render(area, buf);
}
