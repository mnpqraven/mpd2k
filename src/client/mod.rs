use crate::backend::library::create_source;
use crate::error::AppError;
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use std::{path::PathBuf, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::mpsc;

pub mod library;
pub mod mpd;

pub trait PlayableAudio {
    fn path(&self) -> PathBuf;
}

pub trait Playback {
    fn play(&self, audio: Option<impl PlayableAudio>) -> Result<(), AppError>;
    // TODO: pause
    // stop
    // queue next
    // seek
    fn _play(audio: Option<impl PlayableAudio>) -> Result<(), AppError>;
}

pub trait Toggle {
    fn fade_in_out(&self, duration: Option<Duration>) -> Result<(), AppError>;
    // TODO: set volume
    // set gain
    // set sample rate (only lib ?)
}

#[derive(Debug)]
pub enum PlaybackEvent {
    /// this will clear all current queue and start anew, not actually
    /// playing/resuming, pause state is handled by `PlaybackEvent::Pause`
    Play(String),
    /// this toggles between play and paused state
    Pause,
    Tick,
    AppExit,
}

/// wrapper struct that takes ownership of metadata from other clients so they
/// can drop the mutex guard
pub struct PlaybackServer {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<PlaybackEvent>,
    // Event receiver channel.
    pub receiver: mpsc::UnboundedReceiver<PlaybackEvent>,
    pub sink: SinkArc,
    pub stream: OutputStream,
    pub handle: Handle,
}

pub struct SinkArc(pub Arc<Sink>);
impl SinkArc {
    pub fn arced(&self) -> Arc<Sink> {
        self.0.clone()
    }
}

impl PlaybackServer {
    pub fn new(handle: Handle) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());
        let sink = SinkArc(sink);
        Self {
            sender,
            receiver,
            sink,
            stream,
            handle,
        }
    }

    /// run the audio thread, this should return tick command instead of
    /// blocking the main thread
    pub fn handle_events(&mut self) -> Result<(), AppError> {
        // creates a new sink
        // TODO: if this works we can move sink to its own backend struct to
        // manage

        if let Ok(message) = self.receiver.try_recv() {
            match message {
                PlaybackEvent::Play(path) => {
                    let sink = self.sink.arced();
                    let source = create_source(path).unwrap();
                    sink.clear();
                    sink.append(source);
                    sink.play();
                }
                PlaybackEvent::Pause => {
                    let sink = self.sink.arced();
                    match sink.is_paused() {
                        true => sink.play(),
                        false => sink.pause(),
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn next(&mut self) -> Result<PlaybackEvent, AppError> {
        self.receiver.recv().await.ok_or(AppError::Unimplemented)
    }
}

// TODO: what do we do with trait implementation ?
// impl Playback for PlaybackServer {
//     fn _play(track: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
//         // Get a output stream handle to the default physical sound device
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//         // Load a sound from a file, using a path relative to Cargo.toml
//         // TODO: safe unwrap
//         let file = BufReader::new(File::open(track.unwrap().path()).unwrap());
//         // Decode that sound file into a source
//         let source = Decoder::new(file).unwrap();
//         // TODO: safe unwrap
//         let _duration = source.total_duration().unwrap();

//         let sink = Sink::try_new(&stream_handle)?;
//         sink.append(source);
//         sink.sleep_until_end();
//         // Play the sound directly on the device
//         // stream_handle.play_raw(source.convert_samples())?;
//         // The sound plays in a separate audio thread,
//         // so we need to keep the main thread alive while it's playing.
//         // std::thread::sleep(duration);

//         Ok(())
//     }
//     fn play(&self, track: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
//         todo!()
//     }
// }
