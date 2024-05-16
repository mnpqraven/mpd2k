use crate::backend::library::create_source;
use crate::backend::library::types::AudioTrack;
use crate::error::AppError;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::sync::Arc;
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tracing::info;

#[derive(Debug, PartialEq)]
pub enum AppToPlaybackEvent {
    /// path to the audio file
    ///
    /// this will clear all current queue and start anew, not actually
    ///
    /// playing/resuming, pause state is handled by `PlaybackEvent::Pause`
    /// bool indicates whether or not the sink plays instantly
    SetQueue(Arc<[AudioTrack]>),
    AppendQueue(Arc<[AudioTrack]>),
    /// no action if already playing
    Play,
    /// this toggles between play and paused state
    TogglePause,
    /// the play/pause status of the sink
    /// `true` denotes track is playing
    PlayStatus,
    Tick,
    VolumeUp,
    VolumeDown,

    RequestDuration,
}

#[derive(Debug)]
pub enum PlaybackToAppEvent {
    CurrentDuration(i64),
}

/// A server that handles a sink for playback and listens to signals from the
/// client
pub struct PlaybackServer {
    /// Event sender channel.
    pub pb_tx: mpsc::UnboundedSender<AppToPlaybackEvent>,
    /// Global sink that manages playback
    pub sink: SinkArc,
    /// Stream for the sink, this must have a longer lifetime than the sink
    /// itself
    _stream: OutputStream,
    /// App sender channel.
    pub app_tx: UnboundedSender<PlaybackToAppEvent>,
}

pub struct SinkArc(pub Arc<Sink>);
impl SinkArc {
    fn new(stream: &OutputStreamHandle) -> Self {
        Self(Arc::new(
            Sink::try_new(stream).expect("should always have at least one device"),
        ))
    }

    pub fn arced(&self) -> Arc<Sink> {
        self.0.clone()
    }
}

impl PlaybackServer {
    pub fn new_expr(
        pb_tx: &UnboundedSender<AppToPlaybackEvent>,
        app_tx: &UnboundedSender<PlaybackToAppEvent>,
    ) -> Self {
        // NOTE: this stream must not be dropped early
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = SinkArc::new(&stream_handle);

        Self {
            pb_tx: pb_tx.clone(),
            sink,
            _stream: stream,
            app_tx: app_tx.clone(),
        }
    }

    pub fn spawn_listener(&self, mut pb_rx: UnboundedReceiver<AppToPlaybackEvent>) {
        let sink_for_thread = self.sink.arced();
        let tx_inner = self.app_tx.clone();

        tokio::spawn(async move {
            while let Some(message) = pb_rx.recv().await {
                let sink = sink_for_thread.clone();
                handle_message(sink, message, tx_inner.clone())?;
            }
            Ok::<(), AppError>(())
        });
    }
}

fn handle_message(
    sink: Arc<Sink>,
    message: AppToPlaybackEvent,
    tx: UnboundedSender<PlaybackToAppEvent>,
) -> Result<(), AppError> {
    let filter = [AppToPlaybackEvent::PlayStatus, AppToPlaybackEvent::Tick];
    if filter.into_iter().all(|e| e != message) {
        info!(?message);
    }

    match message {
        AppToPlaybackEvent::Play => sink.play(),
        AppToPlaybackEvent::TogglePause => match sink.is_paused() {
            true => sink.play(),
            false => sink.pause(),
        },
        AppToPlaybackEvent::VolumeUp => match sink.volume() {
            0.95.. => sink.set_volume(1.0),
            _ => sink.set_volume(sink.volume() + 0.05),
        },
        AppToPlaybackEvent::VolumeDown => match sink.volume() {
            0.05.. => sink.set_volume(sink.volume() - 0.05),
            _ => sink.set_volume(0.0),
        },
        AppToPlaybackEvent::SetQueue(q_arc) => {
            sink.clear();
            // TODO: perf, reading 20 binaries is very expensive
            for track in q_arc.iter() {
                let source = create_source(track.path.as_ref())?;
                sink.append(source);
            }
        }
        AppToPlaybackEvent::AppendQueue(q_arc) => {
            // TODO: perf, reading 20 binaries is very expensive
            for track in q_arc.iter() {
                let source = create_source(track.path.as_ref())?;
                sink.append(source);
            }
        }
        AppToPlaybackEvent::RequestDuration => {
            tx.send(PlaybackToAppEvent::CurrentDuration(42))?;
            // 42 (send)
        }
        _ => {}
    }
    Ok(())
}
