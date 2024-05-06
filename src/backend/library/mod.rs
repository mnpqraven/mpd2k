use self::types::*;
use crate::{
    backend::utils::is_supported_audio,
    client::library::LibraryClient,
    error::{AppError, LibraryError},
};
use audiotags::Tag;
use rodio::Decoder;
use std::{
    collections::HashSet,
    fmt::Debug,
    fs::File,
    io::BufReader,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Handle, task::JoinSet};
use tracing::{info, instrument};
use walkdir::WalkDir;

pub mod cache;
mod csv;
pub mod hash;
pub mod types;

/// only load path and name
#[instrument(skip(tree_arc))]
pub async fn load_all_tracks_raw<P: AsRef<Path> + Debug>(
    lib_root: P,
    tree_arc: Arc<Mutex<LibraryClient>>,
    hard_update: bool,
) -> Result<(), AppError> {
    if hard_update {
        tree_arc.lock().map(|mut e| e.audio_tracks = vec![])?;
    }

    let library_tree = WalkDir::new(lib_root).follow_links(true);
    for entry in library_tree {
        let entry = entry?;
        let path = entry.path().to_string_lossy().to_string();

        if is_supported_audio(&path) {
            let mut tree_guard = tree_arc.lock()?;
            let trk = AudioTrack::new(path);
            tree_guard.audio_tracks.push(trk);
        }
    }
    // TODO: filtering duplicate paths
    let mut lib = tree_arc.lock()?;
    lib.dedup();

    Ok(())
}

#[instrument(skip_all, ret)]
pub fn load_albums(tree_arc: Arc<Mutex<LibraryClient>>) -> Result<(), AppError> {
    let mut lib = tree_arc.lock()?;

    let mut distict_album_names = HashSet::new();
    for trk in &lib.audio_tracks {
        distict_album_names.insert(trk.album.clone());
    }
    info!(?distict_album_names);

    for album in distict_album_names.iter() {
        let mut tracks: Vec<AudioTrack> = lib
            .audio_tracks
            .iter()
            .filter(|e| e.album == *album)
            .cloned()
            .collect();
        tracks.sort();

        // TODO: mean math
        let (date, album_artist) = tracks
            .first()
            .map(|e| (e.date, e.album_artist.clone()))
            .unwrap();

        // TODO: handle else
        if let Some(album) = album {
            lib.albums.insert(
                AlbumMeta {
                    album_artist,
                    date,
                    name: Some(album.to_string()),
                },
                tracks,
            );
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn inject_metadata(
    tree_arc: Arc<Mutex<LibraryClient>>,
    handle: Handle,
) -> Result<(), AppError> {
    let tracks = tree_arc
        .clone()
        .lock()
        .map(|lib| lib.audio_tracks.clone())?;

    let mut join_set = JoinSet::new();

    for (index, track) in tracks.into_iter().enumerate() {
        let arced = tree_arc.clone();
        let _ = join_set.spawn_on(
            async move {
                let trk = read_tag(track.path.clone()).unwrap();
                {
                    let mut guard = arced.lock().unwrap();
                    let track = guard
                        .audio_tracks
                        .get_mut(index)
                        .expect("index and size should stay unchanged");
                    *track = trk;
                }
            },
            &handle,
        );
    }
    while let Some(t) = join_set.join_next().await {
        let () = t.unwrap();
    }

    Ok(())
}

/// This function is not cheap, running in parallel is recommended
#[instrument]
fn read_tag(path: String) -> Result<AudioTrack, LibraryError> {
    let tag = Tag::new().read_from_path(&path)?;
    let name = tag.title().unwrap_or_default().to_string();
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
        date: SomeAlbumDate(date),
        album,
        album_artist,
        binary_hash: None,
    };
    Ok(track)
}

pub fn create_source<P: AsRef<Path>>(path: P) -> Result<Decoder<BufReader<File>>, AppError> {
    let file = BufReader::new(File::open(path)?);
    let source = Decoder::new(file).map_err(LibraryError::DecoderError)?;
    Ok(source)
}

#[cfg(test)]
mod tests {}
