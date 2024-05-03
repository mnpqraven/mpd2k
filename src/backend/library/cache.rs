use crate::backend::library::{
    hash::{hash_file, HashKind},
    sort_library, AlbumDate, AudioTrack,
};
use crate::backend::utils::empty_to_option;
use crate::dotfile::DotfileSchema;
use crate::error::AppError;
use csv::{StringRecord, Writer};
use std::path::Path;
use std::sync::Arc;
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
    tracks_lock: Arc<tokio::sync::Mutex<Vec<AudioTrack>>>,
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
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(DotfileSchema::cache_path().unwrap())
        .map_err(|_| AppError::BadConfig)
        .unwrap();
    // let writer = Arc::new(Mutex::new(writer));

    let mut futs = JoinSet::new();

    let tracks = tracks_lock.lock().await;
    for track in tracks.iter() {
        let track = track.clone();
        // let writer_inner = writer.clone();
        futs.spawn_on(async move { track_to_hashed_rec(track) }, &handle);
    }
    drop(tracks);

    // NOTE: probably better to use `&mut [AudioTracks]` and update hash inside
    // that
    let mut sorted_tracks = tracks_lock.lock().await;

    // TODO: clean up here ?
    while let Some(e) = futs.join_next().await {
        let hashed_trk = e.unwrap().unwrap();
        // TODO: perf
        let str_rec = StringRecord::from(hashed_trk.to_vec());
        let hashed_trk: AudioTrack = AudioTrack::try_from(str_rec)?;
        sorted_tracks.push(hashed_trk);
    }

    // sorting by hash
    sorted_tracks.sort_by(|a, b| a.binary_hash.cmp(&b.binary_hash));
    sorted_tracks.dedup_by(|a, b| a.binary_hash == b.binary_hash);

    for track in sorted_tracks.iter() {
        let rec: StringRecord = track.try_into().unwrap();
        writer.write_record(&rec)?;
    }

    // resort lib
    sort_library(&mut sorted_tracks)?;

    info!("update_cache complete");
    Ok(())
}

/// this function handles hash lookup return the hashed record(for csv format)
fn track_to_hashed_rec(
    track: AudioTrack,
    // writer: Arc<Mutex<Writer<File>>>,
) -> Result<[String; 8], AppError> {
    let hash = match &track.binary_hash {
        Some(hash) => Some(hash.to_string()),
        None => hash_file(&track.path, HashKind::XxHash).ok(),
    };

    if let Some(hash) = hash {
        let record = as_record(hash, &track);
        return Ok(record);
    }
    Err(AppError::Unimplemented)
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
impl TryFrom<&AudioTrack> for StringRecord {
    type Error = AppError;

    /// TODO: perf
    fn try_from(track: &AudioTrack) -> Result<Self, Self::Error> {
        Ok(Self::from(vec![
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
            track.binary_hash.clone().unwrap_or_default(),
        ]))
    }
}
