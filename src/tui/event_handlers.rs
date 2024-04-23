use crate::error::AppError;
use crossterm::event::{self, KeyCode, KeyEventKind};

#[derive(Debug, Clone)]
pub enum KeyAction {
    Quit,
    // special ops, do nothing
    Nul,
}

pub struct KeyEventHandler {
    // keys, action
    events: Vec<(char, KeyAction)>,
}

impl KeyEventHandler {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn register(&mut self, key: char, action: KeyAction) -> &mut Self {
        // TODO: check multiple declarations
        self.events.push((key, action));
        self
    }

    pub fn init() -> Result<KeyAction, AppError> {
        Self::new().register('q', KeyAction::Quit).listen()
    }

    /// Result<noop: bool>
    /// if `op` is false then nothing happens on parent loop
    /// `op` == true denotes event caught an action
    /// NOTE: expand bool when we needs different response actions in `main.rs`
    fn listen(&self) -> Result<KeyAction, AppError> {
        for (key_code, action) in &self.events {
            // 16ms = 60fps
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(*key_code) {
                        return Ok(action.clone());
                    }
                }
            }
        }
        Ok(KeyAction::Nul)
    }
}
