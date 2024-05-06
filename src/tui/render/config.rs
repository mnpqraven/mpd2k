use crate::client::PlayableClient;
use crate::tui::app::AppState;
use ratatui::widgets::*;
use ratatui::{buffer::Buffer, layout::Rect};

#[allow(non_snake_case)]
pub fn ConfigContainer<Client>(_app: &AppState<Client>, area: Rect, buf: &mut Buffer)
where
    Client: PlayableClient,
{
    Block::new().borders(Borders::all()).render(area, buf);
}
