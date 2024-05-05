use ratatui::{prelude::*, widgets::StatefulWidget};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use std::{fmt::Debug, path::Path};
use tracing::{info, instrument};

#[derive(Default)]
pub struct ImageState(pub Option<Box<dyn StatefulProtocol>>);

impl Debug for ImageState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ImageState").finish()
    }
}

impl ImageState {
    // TODO: cache/memoization ?
    #[instrument]
    pub fn update<P: AsRef<Path> + Debug>(&mut self, path: P) {
        info!(?path);
        let mut picker = Picker::new((8, 12));
        picker.guess_protocol();

        info!(?picker.protocol_type);

        let dyn_img = image::io::Reader::open(path).unwrap().decode().unwrap();

        let img = picker.new_resize_protocol(dyn_img);
        self.0 = Some(img)
    }

    pub fn unset(&mut self) {
        self.0 = None
    }
}

#[derive(Default)]
pub struct Image {}

impl Image {
    pub fn new() -> Self {
        Self {}
    }
    pub fn create_state<P: AsRef<Path>>(path: P) -> Box<dyn StatefulProtocol> {
        let mut picker = Picker::new((8, 12));
        picker.guess_protocol();
        let dyn_img = image::io::Reader::open(path).unwrap().decode().unwrap();

        picker.new_resize_protocol(dyn_img)
    }
}

impl StatefulWidget for Image {
    type State = ImageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.0.is_some() {
            // safe unwrap
            StatefulImage::new(None).render(area, buf, state.0.as_mut().unwrap())
        }
    }
}
