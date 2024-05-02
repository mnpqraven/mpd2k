use crate::backend::library::{
    hash::{hash_file, HashKind},
    sort_library, AlbumDate, AudioTrack,
};
use crate::backend::utils::empty_to_option;
use crate::dotfile::DotfileSchema;
use crate::error::AppError;
use csv::{StringRecord, Writer};
use std::path::Path;
use std::{
    fs::File,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Handle, task::JoinSet};
use tracing::info;

/// try to read from csv cache, else load directly from dir
pub fn try_load_cache<P: AsRef<Path>>(path: P) -> Result<Vec<AudioTrack>, AppError> {
    info!("try_load_cache");
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(path)?;
    let mut tracks = rdr
        .records()
        .flat_map(|record| record?.try_into())
        .collect::<Vec<AudioTrack>>();
    sort_library(&mut tracks)?;
    Ok(tracks)
}

/// this will hash the file if hash is not present
pub async fn try_write_cache_parallel<P: AsRef<Path>>(
    cache_path: P,
    tracks: &[AudioTrack],
    handle: Handle,
) -> Result<(), AppError> {
    // force full scan
    // TODO: does not force full scan, but check hash of existing file
    // if match then go next line
    // if mismatch then replace line with new hash + info
    if cache_path.as_ref().exists() {
        info!("removing cache file");
        tokio::fs::remove_file(&cache_path).await?;
    }

    // writer
    // NOTE: this needs to be a single writer to keep track of the file index
    // pass around threads using usual Arc<Mutex<T>>
    let writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(DotfileSchema::cache_path().unwrap())
        .map_err(|_| AppError::BadConfig)
        .unwrap();
    let writer = Arc::new(Mutex::new(writer));

    let mut futs = JoinSet::new();
    for track in tracks {
        let track = track.clone();
        let writer_inner = writer.clone();
        futs.spawn_on(async move { write_fn(track, writer_inner) }, &handle);
    }

    while let Some(item) = futs.join_next().await {
        let () = item.unwrap();
    }

    info!("update_cache complete");
    Ok(())
}

/// this will hash the file if hash is not present
pub async fn try_write_cache_serial<P: AsRef<Path>>(
    cache_path: P,
    tracks: &[AudioTrack],
    _handle: Handle,
) -> Result<(), AppError> {
    // force full scan
    // TODO: does not force full scan, but check hash of existing file
    // if match then go next line
    // if mismatch then replace line with new hash + info
    if cache_path.as_ref().exists() {
        info!("removing cache file");
        tokio::fs::remove_file(&cache_path).await?;
    }

    // writer
    // NOTE: this needs to be a single writer to keep track of the file index
    // pass around threads using usual Arc<Mutex<T>>
    let writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(DotfileSchema::cache_path().unwrap())
        .map_err(|_| AppError::BadConfig)
        .unwrap();
    let writer = Arc::new(Mutex::new(writer));

    for track in tracks {
        let track = track.clone();
        let writer_inner = writer.clone();
        write_fn(track, writer_inner);
    }

    info!("update_cache complete");
    Ok(())
}

/// this function handles hash lookup and writing the track metadata to the csv
/// file
fn write_fn(track: AudioTrack, writer: Arc<Mutex<Writer<File>>>) {
    let hash = match &track.binary_hash {
        Some(hash) => Some(hash.to_string()),
        None => hash_file(&track.path, HashKind::XxHash).ok(),
    };

    if let Some(hash) = hash {
        let record = as_record(hash, &track);
        let _ = writer.lock().map(|mut writer| {
            writer.write_record(record)?;
            writer.flush()?;
            info!("wrote hash for {}", track.name);
            Ok::<(), AppError>(())
        });
    }
}

fn as_record(hash: String, track: &AudioTrack) -> [String; 8] {
    [
        track.name.to_string(),
        track.path.to_string(),
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
    ]
}

impl TryFrom<StringRecord> for AudioTrack {
    type Error = AppError;

    fn try_from(record: StringRecord) -> Result<Self, Self::Error> {
        if record.len() != 8 {
            return Err(AppError::CsvParse);
        }

        Ok(AudioTrack {
            name: record[0].to_string(),
            path: record[1].to_string(),
            track_no: empty_to_option(&record[2]),
            artist: empty_to_option(&record[3]),
            album: empty_to_option(&record[4]),
            album_artist: empty_to_option(&record[5]),
            date: AlbumDate::parse(&record[6]),
            binary_hash: empty_to_option(&record[7]),
        })
    }
}
