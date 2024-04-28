use crate::{
    backend::utils::is_supported_audio,
    client::{library::LibraryClient, PlayableAudio},
    dotfile::DotfileSchema,
    error::AppError,
};
use audiotags::Tag;
use rodio::Decoder;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::info;
use walkdir::WalkDir;

pub mod cache;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
}

impl PlayableAudio for &AudioTrack {
    fn path(&self) -> PathBuf {
        PathBuf::from(self.path.clone())
    }
}

pub fn load_all_tracks_incremental(
    config: &DotfileSchema,
    tree_arc: Arc<Mutex<LibraryClient>>,
) -> Result<Vec<AudioTrack>, AppError> {
    let root = config.library_root()?;

    let library_tree = WalkDir::new(root).follow_links(true);

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
            // most expensive work
            let tag = Tag::new().read_from_path(path.clone())?;
            let name = tag.title().unwrap_or(&filename).to_string();
            let album = tag.album_title().map(|e| e.to_owned());
            let artist = tag.artist().map(|e| e.to_owned());
            let date  = tag.date();
            info!(?date);
            let album_artist = tag.album_artist().map(|e| e.to_owned());
            let track_no = tag.track_number();

            let track = AudioTrack {
                name,
                path,
                track_no,
                // _meta: entry,
                artist,
                album,
                album_artist,
            };

            // we only lock after the tags lookup is completed
            let mut tree_guard = tree_arc.lock()?;
            tree_guard.audio_tracks.push(track);
            // sort after every album
            if current_dir.as_path().ne(entry.path()) {
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
    let source = Decoder::new(file).unwrap();
    Ok(source)
}

#[cfg(test)]
mod tests {}
