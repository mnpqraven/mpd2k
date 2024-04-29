use std::path::Path;

use crate::backend::library::hash::{hash_file, HashKind};
use crate::backend::library::{AlbumDate, AudioTrack};
use crate::backend::utils::empty_to_option;
use crate::error::AppError;
use tracing::info;

/// try to read from csv cache, else load directly from dir
pub fn try_load_cache<P: AsRef<Path>>(path: P) -> Result<Vec<AudioTrack>, AppError> {
    info!("try_load_cache");
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let tracks = rdr
        .records()
        .flat_map(|record| {
            let record = record?;

            Ok::<AudioTrack, AppError>(AudioTrack {
                name: record[0].to_string(),
                path: record[1].to_string(),
                track_no: empty_to_option(&record[2]),
                artist: empty_to_option(&record[3]),
                album: empty_to_option(&record[4]),
                album_artist: empty_to_option(&record[5]),
                date: AlbumDate::parse(&record[6]),
                binary_hash: empty_to_option(&record[7]),
            })
        })
        .collect::<Vec<AudioTrack>>();
    Ok(tracks)
}

/// this will hash the file if hash is not present
pub async fn try_write_cache<P: AsRef<Path>>(
    cache_path: P,
    tracks: &[AudioTrack],
) -> Result<(), AppError> {
    // force full scan
    // TODO: does not force full scan, but check hash of existing file
    // if match then go next line
    // if mismatch then replace line with new hash + info
    if cache_path.as_ref().exists() {
        info!("removing cache file");
        tokio::fs::remove_file(&cache_path).await?;
    }

    // write
    let mut writer = csv::WriterBuilder::new()
        .from_path(cache_path)
        .map_err(|_| AppError::BadConfig)?;

    tracks.iter().for_each(|track| {
        info!("hashing {}", track.path);

        let mut write_action = |hash: String| {
            let as_bytes = &[
                track.name.clone(),
                track.path.clone(),
                track.track_no.map(|no| no.to_string()).unwrap_or_default(),
                track.artist.clone().unwrap_or_default(),
                track.album.clone().unwrap_or_default(),
                track.album_artist.clone().unwrap_or_default(),
                track
                    .date
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
                hash,
            ];
            writer.write_record(as_bytes).unwrap();
        };

        // don't do hash if hash is already present, whether or not if it's incorrect
        let hash = match &track.binary_hash {
            Some(hash) => Some(hash.to_string()),
            None => hash_file(&track.path, HashKind::Murmur).ok(),
        };

        if let Some(hash) = hash {
            write_action(hash);
        }
    });
    info!("update_cache complete");
    Ok(())
}
