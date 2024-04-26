use crate::error::AppError;
use crate::tui::types::AppState;
use core::panic;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};
use std::{path::PathBuf, time::Duration};
use tokio::sync::mpsc;
use tracing::info;

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

pub enum PlaybackEvent {
    Play,
    Pause,
    Tick,
}

/// wrapper struct that takes ownership of metadata from other clients so they
/// can drop the mutex guard
#[derive(Debug)]
pub struct PlaybackClient {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<PlaybackEvent>,
    // Event receiver channel.
    pub receiver: mpsc::UnboundedReceiver<PlaybackEvent>,
    pub sink: Bruh,
    // playback thread.
    // playback_thread: tokio::task::JoinHandle<()>,
}

pub struct Bruh(pub Arc<Mutex<Sink>>);
impl std::fmt::Debug for Bruh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Bruh").finish()
    }
}

impl PlaybackClient {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // let arced = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
        let arced = Arc::new(Mutex::new(Sink::new_idle().0));
        let sink = Bruh(arced);

        Self {
            sender,
            receiver,
            sink,
        }
    }

    pub async fn next(&mut self) -> Result<PlaybackEvent, AppError> {
        self.receiver.recv().await.ok_or(AppError::Unimplemented)
    }

    pub async fn handle(&mut self, app: &mut AppState) -> Result<(), AppError> {
        match self.receiver.try_recv() {
            Ok(ev) => match ev {
                PlaybackEvent::Play => {
                    info!("play");
                    let sink = self.sink.0.clone();
                    let lib = app.library_client.clone();
                    tokio::spawn(async move {
                        let lib = lib.lock().unwrap();
                        let tui_state = lib.tui_state.lock().unwrap();

                        let index = tui_state.selected().unwrap();
                        let track = lib.audio_tracks.get(index).unwrap().clone();
                        info!(track.path);
                        let file = BufReader::new(File::open(track.path).unwrap());
                        let source = Decoder::new(file).unwrap();
                        // TODO: receiver process sink inside this block
                        // let sink = sink.lock().unwrap();
                        // sink.append(source);
                        // sink.sleep_until_end();

                        let sink = sink.lock().unwrap();
                        // drop all guard
                        drop(tui_state);
                        drop(lib);

                        sink.append(source);
                        sink.play();
                        sink.sleep_until_end();
                    });
                }
                PlaybackEvent::Pause => {
                    info!("pause")
                }
                PlaybackEvent::Tick => {}
            },
            _ => {
                info!("none")
            }
        }
        Ok(())
    }
}

pub fn handle_playback_event(
    // app_state: Arc<Mutex<AppState>>, event: PlaybackEvent
    cmd_tx: mpsc::UnboundedSender<PlaybackEvent>,
    mut cmd_rx: mpsc::UnboundedReceiver<PlaybackEvent>,
) -> Result<(), AppError> {
    while let Some(event) = cmd_rx.blocking_recv() {
        // match event {
        //     PlaybackEvent::Play => info!("executing play"),
        //     PlaybackEvent::Pause => info!("executing pause"),
        // }
    }
    // let sink = app_state.playback_client.sink.0.clone();
    // let lib_arc = app_state.library_client.clone();
    // match event {
    //     PlaybackEvent::Play => {
    //         tokio::spawn(async move {
    //             let lib = lib_arc.lock().unwrap();
    //             let tui_state = lib.tui_state.lock().unwrap();
    //
    //             let index = tui_state.selected().unwrap();
    //             let track = lib.audio_tracks.get(index).unwrap().clone();
    //             let file = BufReader::new(File::open(track.path).unwrap());
    //             let source = Decoder::new(file).unwrap();
    //             // TODO: receiver process sink inside this block
    //             // let sink = sink.lock().unwrap();
    //             // sink.append(source);
    //             // sink.sleep_until_end();
    //         });
    //     }
    //     PlaybackEvent::Pause => todo!(),
    // }
    Ok(())
}

impl Playback for PlaybackClient {
    fn _play(track: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        // TODO: safe unwrap
        let file = BufReader::new(File::open(track.unwrap().path()).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // TODO: safe unwrap
        let _duration = source.total_duration().unwrap();

        let sink = Sink::try_new(&stream_handle)?;
        sink.append(source);
        sink.sleep_until_end();
        // Play the sound directly on the device
        // stream_handle.play_raw(source.convert_samples())?;
        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        // std::thread::sleep(duration);

        Ok(())
    }
    fn play(&self, track: Option<impl PlayableAudio>) -> Result<(), crate::error::AppError> {
        todo!()
    }
}
