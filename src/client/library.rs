use std::{fs::File, io::BufReader};

use super::Playback;
use crate::backend::library::AudioTrack;
use rodio::{Decoder, OutputStream, Source};

pub struct LibraryClient {
    tracks: Vec<AudioTrack>,
}

impl Playback for LibraryClient {
    fn play(&self) -> Result<(), crate::error::AppError> {
        let track = self.tracks.first().unwrap();

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
