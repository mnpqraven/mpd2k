use crate::backend::library::create_source;
use crate::backend::library::types::AudioTrack;
use crate::error::AppError;
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
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
    Tick,
}

/// wrapper struct that takes ownership of metadata from other clients so they
/// can drop the mutex guard
pub struct PlaybackServer {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<AppToPlaybackEvent>,
    /// Event receiver channel.
    // pub receiver: mpsc::UnboundedReceiver<AppToPlaybackEvent>,
    /// Global sink that manages playback
    pub sink: SinkArc,
    pub stream: OutputStream,
    pub app_tx: UnboundedSender<PlaybackToAppEvent>,
}

pub struct SinkArc(pub Arc<Sink>);
impl SinkArc {
    pub fn arced(&self) -> Arc<Sink> {
        self.0.clone()
    }
}

// TODO: need to impl message passing the other way (from this to AppState)
impl PlaybackServer {
    pub fn new_expr(
        tx: UnboundedSender<AppToPlaybackEvent>,
        mut rx: UnboundedReceiver<AppToPlaybackEvent>,
        app_tx: UnboundedSender<PlaybackToAppEvent>,
    ) -> (Self, mpsc::UnboundedSender<AppToPlaybackEvent>) {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());
        let sink = SinkArc(sink);
        let sink_for_thread = sink.arced();
        // let (tx, mut rx) = mpsc::unbounded_channel::<AppToPlaybackEvent>();

        let tx_inner = app_tx.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let sink = sink_for_thread.clone();
                handle_message(sink, message, tx_inner.clone())?;
            }
            Ok::<(), AppError>(())
        });

        let res = Self {
            sender: tx.clone(),
            sink,
            stream,
            app_tx,
        };
        (res, tx)
    }

    /// run the audio thread, this should return tick command instead of
    /// blocking the main thread
    pub fn handle_events(&mut self) -> Result<(), AppError> {
        // creates a new sink
        // TODO: if this works we can move sink to its own backend struct to
        // manage

        // FIX: messages is missing
        // if let Ok(message) = self.receiver.try_recv() {
        //     let sink = self.sink.arced();
        //
        //     if message != AppToPlaybackEvent::PlayStatus && message != AppToPlaybackEvent::Tick {
        //         info!(?message);
        //     }
        //
        //     match message {
        //         AppToPlaybackEvent::Play => sink.play(),
        //         AppToPlaybackEvent::TogglePause => match sink.is_paused() {
        //             true => sink.play(),
        //             false => sink.pause(),
        //         },
        //         AppToPlaybackEvent::VolumeUp => {
        //             // TODO: works but bad practice, create a different init block instead
        //             // self.sender
        //             //     .send(PlaybackToAppEvent::CurrentDuration(1000))?;
        //
        //             match sink.volume() {
        //                 0.95.. => sink.set_volume(1.0),
        //                 _ => sink.set_volume(sink.volume() + 0.05),
        //             }
        //         }
        //         AppToPlaybackEvent::VolumeDown => match sink.volume() {
        //             0.05.. => sink.set_volume(sink.volume() - 0.05),
        //             _ => sink.set_volume(0.0),
        //         },
        //         AppToPlaybackEvent::SetQueue(q_arc) => {
        //             sink.clear();
        //             // TODO: perf, reading 20 binaries is very expensive
        //             for track in q_arc.iter() {
        //                 let source = create_source(track.path.as_ref())?;
        //                 sink.append(source);
        //             }
        //         }
        //         AppToPlaybackEvent::AppendQueue(q_arc) => {
        //             // TODO: perf, reading 20 binaries is very expensive
        //             for track in q_arc.iter() {
        //                 let source = create_source(track.path.as_ref())?;
        //                 sink.append(source);
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        Ok(())
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
