use self::events::{AppToPlaybackEvent, PlaybackToAppEvent};
use crate::backend::library::types::{AlbumMeta, AudioTrack, CurrentTrack, RepeatMode};
use crate::{error::AppError, tui::app::TuiState};
use ratatui::widgets::TableState;
use std::collections::BTreeMap;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, TryLockResult};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod events;
pub mod library;
pub mod mpd;

pub enum ClientKind {
    Library,
    Mpd,
}

pub trait PlayableClient {
    fn new(
        playback_tx: UnboundedSender<AppToPlaybackEvent>,
        // playback_tx: UnboundedSender<AppToPlaybackEvent>,
        // playback_rx: UnboundedReceiver<PlaybackToAppEvent>,
    ) -> Self;
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError>;

    fn current_track(&self) -> Option<&CurrentTrack>;

    fn loading(&self) -> bool;

    fn audio_tracks(&self) -> Vec<&AudioTrack>;
    fn albums(&self) -> &BTreeMap<AlbumMeta, Vec<AudioTrack>>;

    fn select_next_track(&self, table_state: &mut TuiState) -> Result<(), AppError>;
    fn select_prev_track(&self, table_state: &mut TuiState) -> Result<(), AppError>;
    fn select_first_track(&self, table_state: &mut TuiState) -> Result<(), AppError>;
    fn select_last_track(&self, table_state: &mut TuiState) -> Result<(), AppError>;

    /// updates the last_album + image if necessary
    fn check_image(&self, tui_state: &mut TuiState) -> Result<(), AppError>;

    fn pause_unpause(&self);
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<Self>>>, hard_update: bool);

    fn volume_percentage(&self) -> u8;
    fn volume_up(&mut self);
    fn volume_down(&mut self);

    fn generate_random_queue(&self) -> Result<Arc<[AudioTrack]>, AppError>;

    fn get_play(&self) -> bool;
    fn get_repeat(&self) -> RepeatMode;
    fn get_shuffle(&self) -> bool;
    fn cycle_repeat(&mut self);
    fn toggle_shuffle(&mut self);

    // TODO:
    // stop
    // queue next
    // seek
    fn kind(&self) -> ClientKind;
    fn cleanup(self);
}

#[derive(Debug)]
pub struct PlaybackClient<Client>
where
    Client: PlayableClient,
{
    inner: Arc<Mutex<Client>>,
    // pub sender: UnboundedSender<AppToPlaybackEvent>,
    // pub receiver: UnboundedReceiver<PlaybackToAppEvent>,
}

impl<Client> PlaybackClient<Client>
where
    Client: PlayableClient,
{
    pub fn new(
        playback_tx: UnboundedSender<AppToPlaybackEvent>,
        // playback_rx: UnboundedReceiver<PlaybackToAppEvent>,
    ) -> Self {
        let inner = Arc::new(Mutex::new(Client::new(playback_tx)));
        Self {
            inner,
            // sender: playback_tx,
            // receiver: playback_rx,
        }
    }

    pub fn from_client(client: Client) -> Self {
        Self {
            inner: Arc::new(Mutex::new(client)),
        }
    }

    /// lock of the playback client
    /// this function blocks until the lock is free
    pub fn get(&self) -> LockResult<MutexGuard<'_, Client>> {
        self.inner.lock()
    }

    /// lock of the playback client
    /// this function does not block
    pub fn try_get(&self) -> TryLockResult<MutexGuard<'_, Client>> {
        self.inner.try_lock()
    }

    /// Triggers the update on the audio list,
    ///
    /// For library the directory is fully loaded unhashed then a
    /// hashing worker is queued in the background
    pub fn update_lib(&mut self, hard_update: bool) -> Result<(), AppError> {
        let arced = self.inner.clone();
        let mut inner = self.inner.lock()?;
        inner.update_lib(Some(arced), hard_update);
        Ok(())
    }

    // consume self, returning inner struct data
    pub fn teardown(self) -> Result<(), AppError> {
        Arc::into_inner(self.inner)
            .map(|e| e.into_inner())
            .ok_or(AppError::BadUnwrap(Some("bad arc consume".into())))??
            .cleanup();

        Ok(())
    }
}
