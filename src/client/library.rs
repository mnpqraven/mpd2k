use super::Playback;
use crate::backend::library::{cache::try_load_cache, AudioTrack};
use ratatui::widgets::TableState;
use rodio::{Decoder, OutputStream, Source};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct LibraryClient {
    pub audio_tracks: Vec<AudioTrack>,
    pub selected_track_index: u32,
    pub tui_state: Arc<Mutex<TableState>>,
}

impl Default for LibraryClient {
    fn default() -> Self {
        Self {
            audio_tracks: try_load_cache().unwrap_or_default(),
            selected_track_index: Default::default(),
            tui_state: Default::default(),
        }
    }
}

impl Playback for LibraryClient {
    fn play(&self) -> Result<(), crate::error::AppError> {
        let track = self.audio_tracks.first().unwrap();

        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open(&track.path).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // TODO: safe unwrap
        let duration = source.total_duration().unwrap();

        // Play the sound directly on the device
        stream_handle.play_raw(source.convert_samples())?;
        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        std::thread::sleep(duration);

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
