use crate::backend::library::create_source;
use crate::error::AppError;
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PlaybackEvent {
    /// path to the audio file
    ///
    /// this will clear all current queue and start anew, not actually
    ///
    /// playing/resuming, pause state is handled by `PlaybackEvent::Pause`
    Play(String),
    /// this toggles between play and paused state
    Pause,
    Tick,
    VolumeUp,
    VolumeDown,
}

/// wrapper struct that takes ownership of metadata from other clients so they
/// can drop the mutex guard
pub struct PlaybackServer {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<PlaybackEvent>,
    /// Event receiver channel.
    pub receiver: mpsc::UnboundedReceiver<PlaybackEvent>,
    /// Global sink that manages playback
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
            let sink = self.sink.arced();

            match message {
                PlaybackEvent::Play(path) => {
                    let source = create_source(path).unwrap();
                    sink.clear();
                    sink.append(source);
                    sink.play();
                }
                PlaybackEvent::Pause => match sink.is_paused() {
                    true => sink.play(),
                    false => sink.pause(),
                },
                PlaybackEvent::VolumeUp => match sink.volume() {
                    0.95.. => sink.set_volume(1.0),
                    _ => sink.set_volume(sink.volume() + 0.05),
                },
                PlaybackEvent::VolumeDown => match sink.volume() {
                    0.05.. => sink.set_volume(sink.volume() - 0.05),
                    _ => sink.set_volume(0.0),
                },
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn next(&mut self) -> Result<PlaybackEvent, AppError> {
        self.receiver.recv().await.ok_or(AppError::Unimplemented)
    }
}