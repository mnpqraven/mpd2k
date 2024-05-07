use crate::{
    backend::utils::empty_to_option, client::events::PlaybackEvent, dotfile::DotfileSchema,
    error::AppError,
};
use audiotags::TimestampTag;
use chrono::{Datelike, NaiveDate};
use csv::StringRecord;
use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::UnboundedSender,
};

use super::cache::{try_load_cache, try_load_cache_albums};

#[derive(Debug)]
pub struct LibraryClient {
    /// TODO: deprecate
    #[deprecated = "use `albums` field instead"]
    pub audio_tracks: Vec<AudioTrack>,
    pub albums: BTreeMap<AlbumMeta, Vec<AudioTrack>>,
    pub current_track: Option<CurrentTrack>,
    pub playback_tx: UnboundedSender<PlaybackEvent>,
    /// this indicates which context the app is in to handle different case of
    /// keyboard event
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub volume: f32,
    /// indicates the loading state of fetching audio tracks and caching if
    /// using file library
    pub loading: bool,
    pub hashing_rt: Runtime,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum RepeatMode {
    #[default]
    Off,
    One,
    All,
}

#[derive(Debug)]
pub struct CurrentTrack {
    pub data: AudioTrack,
    pub duration: Duration,
}

#[allow(dead_code)]
#[derive(Debug, strum::Display)]
pub enum HashKind {
    Sha256,
    Murmur,
    XxHash,
}

// NOTE: keep expanding this or migrate to album(outer struct) > tracks(inner struct)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AudioTrack {
    pub name: Arc<str>,
    pub path: Arc<str>,
    pub artist: Option<Arc<str>>,
    pub album: Option<Arc<str>>,
    pub album_artist: Option<Arc<str>>,
    pub track_no: Option<u16>,
    pub date: SomeAlbumDate,
    pub binary_hash: Option<Arc<str>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AlbumMeta {
    pub album_artist: Option<Arc<str>>,
    pub date: SomeAlbumDate,
    pub name: Option<Arc<str>>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SomeAlbumDate(pub Option<AlbumDate>);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct AlbumDate {
    // at year is always `Some`, if we can't parse year then the whole struct is safe to be `None`
    pub year: u32,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl AudioTrack {
    const CSV_COLS: usize = 8;

    // TODO: perf + unicode check
    pub fn new<P: AsRef<Path> + Into<Arc<str>>>(path: P) -> Self {
        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        Self {
            name: name.into(),
            path: path.into(),
            artist: None,
            album: None,
            album_artist: None,
            track_no: None,
            date: SomeAlbumDate(None),
            binary_hash: None,
        }
    }

    pub fn to_record(&self) -> StringRecord {
        let as_vec: &[String; Self::CSV_COLS] = &[
            self.name.to_string(),
            self.path.to_string(),
            self.track_no.map(|no| no.to_string()).unwrap_or_default(),
            self.artist.as_ref().map(Arc::to_string).unwrap_or_default(),
            self.album.as_ref().map(Arc::to_string).unwrap_or_default(),
            self.album_artist
                .as_ref()
                .map(Arc::to_string)
                .unwrap_or_default(),
            self.date
                .0
                .as_ref()
                .map(AlbumDate::to_string)
                .unwrap_or_default(),
            // ----
            // NOTE: ALWAYS PUT THIS LAST FOR `record_hash`
            self.binary_hash
                .as_ref()
                .map(Arc::to_string)
                .unwrap_or_default(),
        ];
        StringRecord::from(as_vec.as_slice())
    }

    pub fn from_record(record: StringRecord) -> Result<Self, AppError> {
        if record.len() != Self::CSV_COLS {
            return Err(AppError::CsvParse);
        }
        let track = AudioTrack {
            name: record[0].into(),
            path: record[1].into(),
            track_no: empty_to_option(&record[2]),
            artist: empty_to_option::<String>(&record[3]).map(Into::into),
            album: empty_to_option::<String>(&record[4]).map(Into::into),
            album_artist: empty_to_option::<String>(&record[5]).map(Into::into),
            date: SomeAlbumDate(AlbumDate::parse(TimestampTag::Unknown(
                record[6].to_string(),
            ))),
            binary_hash: empty_to_option::<String>(&record[7]).map(Into::into),
        };

        Ok(track)
    }

    pub fn try_cover_path(&self) -> Option<PathBuf> {
        let track_path = PathBuf::from(self.path.as_ref());
        let dir = track_path.parent();
        if let Some(dir) = dir {
            let img_paths: Vec<PathBuf> = read_dir(dir)
                .unwrap()
                .filter(|e| {
                    let path = e.as_ref().unwrap().path();
                    ["png", "jpg"]
                        .into_iter()
                        .any(|ext| ext == path.extension().unwrap())
                })
                .map(|e| e.unwrap().path())
                .collect();
            return img_paths.first().cloned();
        }
        None
    }
}

impl AlbumDate {
    pub fn parse(text: TimestampTag) -> Option<Self> {
        match text {
            TimestampTag::Id3(_) => todo!(),
            TimestampTag::Unknown(text) =>
            // TODO: more formats
            {
                match NaiveDate::parse_from_str(&text, "%Y.%m.%d") {
                    Ok(s) => Some(Self {
                        year: s.year() as u32,
                        month: Some((s.month0() + 1) as u8),
                        day: Some((s.day0() + 1) as u8),
                    }),
                    Err(_) => None,
                }
            }
        }
    }
}

impl LibraryClient {
    pub fn new(playback_tx: UnboundedSender<PlaybackEvent>) -> Self {
        let audio_tracks = try_load_cache(DotfileSchema::cache_path().unwrap()).unwrap_or_default();
        Self {
            audio_tracks: audio_tracks.clone(),
            loading: false,
            volume: 1.0,
            current_track: None,
            hashing_rt: Builder::new_multi_thread()
                // ensure hash is written in reasonable amount of time
                // for 20~50Mb FLACs
                .worker_threads(8)
                .thread_name("hashing-worker")
                .build()
                .expect("Creating a tokio runtime on 12 threads"),
            albums: try_load_cache_albums(audio_tracks),
            playback_tx,
            shuffle: false,
            repeat: Default::default(),
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn cleanup(self) {
        self.hashing_rt.shutdown_background();
    }

    pub fn dedup(&mut self) {
        let path_cmp = |a: &AudioTrack, b: &AudioTrack| a.path.cmp(&b.path);
        self.audio_tracks.sort_by(path_cmp);
        self.audio_tracks.dedup();
        self.audio_tracks.sort();
    }
}
