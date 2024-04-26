use super::{PlayableAudio, Playback};
use crate::backend::library::{cache::try_load_cache, AudioTrack};
use ratatui::widgets::TableState;
use rodio::{Decoder, OutputStream, Source};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};
use tracing::info;

#[derive(Debug)]
pub struct LibraryClient {
    pub audio_tracks: Vec<AudioTrack>,
    pub tui_state: Arc<Mutex<TableState>>,
}

impl Default for LibraryClient {
    fn default() -> Self {
        info!("running default");
        Self {
            audio_tracks: try_load_cache().unwrap_or_default(),
            // selected_track_index: Default::default(),
            tui_state: Default::default(),
        }
    }
}

impl Playback for LibraryClient {
    fn play(&self, track: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        // TODO: safe unwrap
        let file = BufReader::new(File::open(track.unwrap().path()).unwrap());
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

    fn _play(audio: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
