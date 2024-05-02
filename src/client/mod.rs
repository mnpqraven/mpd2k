use self::{events::PlaybackEvent, library::CurrentTrack};
use crate::{backend::library::AudioTrack, error::AppError};
use ratatui::widgets::TableState;
use std::{
    path::PathBuf,
    sync::{Arc, LockResult, Mutex, MutexGuard, TryLockResult},
};
use tokio::sync::mpsc::UnboundedSender;

pub mod events;
pub mod library;
pub mod mpd;

pub enum ClientKind {
    Library,
    Mpd,
}

pub trait PlayableAudio {
    fn path(&self) -> PathBuf;
}

pub trait PlayableClient {
    fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self;
    fn play(&mut self, table_state: &TableState) -> Result<(), AppError>;

    fn current_track(&self) -> Option<&CurrentTrack>;

    fn loading(&self) -> bool;

    fn audio_tracks(&self) -> &[AudioTrack];

    fn select_next_track(&self, table_state: &mut TableState);
    fn select_prev_track(&self, table_state: &mut TableState);
    fn select_last_track(&self, table_state: &mut TableState);
    fn pause_unpause(&self);
    fn update_lib(&mut self, self_arc: Option<Arc<Mutex<Self>>>);

    fn volume_percentage(&self) -> u8;
    fn volume_up(&mut self);
    fn volume_down(&mut self);

    // TODO:
    // stop
    // queue next
    // seek
    fn kind(&self) -> ClientKind;
}

#[derive(Debug)]
pub struct PlaybackClient<Client>
where
    Client: PlayableClient,
{
    pub inner: Arc<Mutex<Client>>,
}

impl<Client> PlaybackClient<Client>
where
    Client: PlayableClient,
{
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        let inner = Arc::new(Mutex::new(Client::new(playback_tx)));
        Self { inner }
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
    pub fn update_lib(&mut self) -> Result<(), AppError> {
        let arced = self.inner.clone();
        let mut inner = self.inner.lock()?;
        inner.update_lib(Some(arced));
        Ok(())
    }
}
