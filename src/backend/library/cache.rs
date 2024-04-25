use crate::{
    backend::{
        library::{load_all_tracks_incremental, AudioTrack},
        utils::empty_to_option,
    },
    client::library::LibraryClient,
    dotfile::DotfileSchema,
    error::AppError,
};
use std::sync::{Arc, Mutex};
use tracing::info;

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

/// TODO: hashing files
/// compare hash to see if a file has changed its metadata and needs to be
/// updated
pub fn update_cache(
    config: &DotfileSchema,
    tree_arc: Arc<Mutex<LibraryClient>>,
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
