use crate::{
    backend::utils::is_supported_audio,
    client::{library::LibraryClient, PlayableAudio},
    error::{AppError, LibraryError},
};
use audiotags::Tag;
use chrono::{Datelike, NaiveDate};
use rodio::Decoder;
use std::{
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::info;
use walkdir::WalkDir;

pub mod cache;
pub mod hash;

// NOTE: keep expanding this or migrate to album(outer struct) > tracks(inner struct)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
    pub date: Option<AlbumDate>,
    pub(self) binary_hash: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AlbumDate {
    // at year is always `Some`, if we can't parse year then the whole struct is safe to be `None`
    pub year: u32,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl AlbumDate {
    fn parse(text: &str) -> Option<Self> {
        // TODO: more formats
        match NaiveDate::parse_from_str(text, "%Y.%m.%d") {
            Ok(s) => Some(Self {
                year: s.year() as u32,
                month: Some((s.month0() + 1) as u8),
                day: Some((s.day0() + 1) as u8),
            }),
            Err(_) => None,
        }
    }
}

impl Display for AlbumDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.year.to_string();
        if let Some(month) = self.month {
            s.push_str(&format!(".{month}"));
        }
        if let Some(day) = self.day {
            s.push_str(&format!(".{day}"));
        }
        write!(f, "{}", s)
    }
}

impl PlayableAudio for &AudioTrack {
    fn path(&self) -> PathBuf {
        PathBuf::from(self.path.clone())
    }
}

pub fn load_all_tracks_unhashed<P: AsRef<Path>>(
    lib_root: P,
    tree_arc: Arc<Mutex<LibraryClient>>,
    hard_update: bool,
) -> Result<Vec<AudioTrack>, AppError> {
    if hard_update {
        tree_arc.lock().map(|mut e| e.audio_tracks = vec![])?;
    }

    let library_tree = WalkDir::new(lib_root).follow_links(true);

    let mut current_dir = PathBuf::new();

    for entry in library_tree {
        let entry = entry?;
        let (filename, path) = (
            entry.file_name().to_string_lossy().to_string(),
            entry.path().to_string_lossy().to_string(),
        );

        if is_supported_audio(&path) {
            // TODO:
            // separate thread
            // 2nd most expensive work
            let track = read_tag(path, &filename)?;

            // we only lock after the tags lookup is completed
            let mut tree_guard = tree_arc.lock()?;

            // FIX: this push is won't erase all the existing tracks and avoid
            // layout shift, but also won't clear out invalid tracks
            tree_guard.audio_tracks.push(track);
            // sort after every album
            if current_dir.as_path().ne(entry.path()) {
                // NOTE: is this actually necessary after we implement
                // appending by folders ?
                sort_library(&mut tree_guard.audio_tracks)?;
                // final dedup or after sort

                current_dir = PathBuf::from(entry.path());
            }
            drop(tree_guard);
        }
    }

    let mut tree_guard = tree_arc.lock()?;
    sort_library(&mut tree_guard.audio_tracks)?;
    tree_guard.audio_tracks.dedup();
    info!(
        "load_all_tracks_incremental len: {}",
        tree_guard.audio_tracks.len()
    );

    Ok(tree_guard.audio_tracks.to_vec())
}

fn read_tag(path: String, filename: &str) -> Result<AudioTrack, LibraryError> {
    let tag = Tag::new().read_from_path(&path)?;
    let name = tag.title().unwrap_or(filename).to_string();
    let album = tag.album_title().map(|e| e.to_owned());
    let artist = tag.artist().map(|e| e.to_owned());
    let date = tag.date_raw().and_then(AlbumDate::parse);
    let album_artist = tag.album_artist().map(|e| e.to_owned());
    let track_no = tag.track_number();

    let track = AudioTrack {
        name,
        path,
        track_no,
        // _meta: entry,
        artist,
        date,
        album,
        album_artist,
        binary_hash: None,
    };
    Ok(track)
}

pub fn sort_library(tracks: &mut [AudioTrack]) -> Result<(), AppError> {
    // album_artist > year > album name > track no
    tracks.sort_unstable_by_key(|item| {
        (
            item.album_artist.clone(),
            // TODO: year here
            item.album.clone(),
            item.track_no,
            item.name.clone(),
        )
    });
    Ok(())
}

pub fn create_source<P: AsRef<Path>>(path: P) -> Result<Decoder<BufReader<File>>, AppError> {
    let file = BufReader::new(File::open(path)?);
    let source = Decoder::new(file).map_err(LibraryError::DecoderError)?;
    Ok(source)
}

#[cfg(test)]
mod tests {}
