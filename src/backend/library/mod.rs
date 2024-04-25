use crate::{
    backend::utils::is_supported_audio, dotfile::DotfileSchema, error::AppError,
    tui::AudioTreeState,
};
use audiotags::Tag;
use std::sync::{Arc, Mutex};
use tracing::info;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
}

pub fn load_all_tracks_incremental(
    config: &DotfileSchema,
    tree_arc: Arc<Mutex<AudioTreeState>>,
) -> Result<Vec<AudioTrack>, AppError> {
    let root = config.library_root()?;

    let library_tree = WalkDir::new(root).follow_links(true);

    for entry in library_tree {
        let entry = entry?;
        let (filename, path) = (
            entry.file_name().to_string_lossy().to_string(),
            entry.path().to_string_lossy().to_string(),
        );

        if is_supported_audio(&path) {
            // TODO:
            // - safe unwrap
            // separate thread
            let tag = Tag::new().read_from_path(path.clone())?;
            let name = tag.title().unwrap_or(&filename).to_string();
            let album = tag.album_title().map(|e| e.to_owned());
            let artist = tag.artist().map(|e| e.to_owned());
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
            let mut tree_guard = tree_arc.lock()?;
            tree_guard.audio_tracks.push(track);
            drop(tree_guard);
        }
    }

    // loading completed, begin sorting
    let mut tree_guard = tree_arc.lock()?;
    sort_library(&mut tree_guard.audio_tracks)?;

    Ok(tree_guard.audio_tracks.to_vec())
}

pub fn sort_library(tracks: &mut [AudioTrack]) -> Result<(), AppError> {
    // TODO: add year sort
    // album_artist > year > album name > track no
    tracks.sort_unstable_by_key(|item| {
        (
            item.album_artist.clone().unwrap_or_default(),
            item.album.clone().unwrap_or_default(),
            item.track_no,
        )
    });
    Ok(())
}

/// try to read from csv cache, else load directly from dir
pub fn try_load_cache() -> Result<Vec<AudioTrack>, AppError> {
    info!("try_load_cache");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(DotfileSchema::cache_path()?)?;
    Ok(rdr
        .records()
        .map(|record| {
            let record = record.unwrap();

            AudioTrack {
                name: record[0].to_string(),
                path: record[1].to_string(),
                track_no: empty_to_option(&record[2]),
                artist: empty_to_option(&record[3]),
                album: empty_to_option(&record[4]),
                album_artist: empty_to_option(&record[5]),
            }
        })
        .collect::<Vec<AudioTrack>>())
}

// convert empty string to None
fn empty_to_option<T: std::str::FromStr + std::default::Default>(text: &str) -> Option<T> {
    match text.is_empty() {
        true => None,
        false => Some(text.parse::<T>().unwrap_or_default()),
    }
}

/// TODO: hashing files
/// compare hash to see if a file has changed its metadata and needs to be
/// updated
pub fn update_cache(
    config: &DotfileSchema,
    tree_arc: Arc<Mutex<AudioTreeState>>,
) -> Result<Vec<AudioTrack>, AppError> {
    info!("update_cache");
    match load_all_tracks_incremental(config, tree_arc) {
        Ok(tracks) => {
            // write
            let cache_path = DotfileSchema::cache_path()?;
            let mut writer = csv::WriterBuilder::new()
                .from_path(cache_path)
                .map_err(|_| AppError::BadConfig)?;

            tracks.iter().for_each(|track| {
                let as_bytes = &[
                    track.name.clone(),
                    track.path.clone(),
                    track.track_no.map(|no| no.to_string()).unwrap_or_default(),
                    track.artist.clone().unwrap_or_default(),
                    track.album.clone().unwrap_or_default(),
                    track.album_artist.clone().unwrap_or_default(),
                ];
                writer.write_record(as_bytes).unwrap();
            });
            Ok(tracks)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(windows)]
    fn display() {
        use super::load_all_tracks;
        use crate::dotfile::DotfileSchema;

        let cfg = DotfileSchema::parse().unwrap();
        let tracks = load_all_tracks(&cfg);
        assert!(tracks.is_ok());
        assert!(!tracks.unwrap().is_empty());
    }
}
