use super::{csv::app_writer_append, hash::hash_file, HashKind};
use crate::{
    backend::library::{
        csv::{app_reader, app_writer_non_append},
        AudioTrack,
    },
    client::library::LibraryClient,
    error::AppError,
};
use csv::StringRecord;
use std::{
    fmt::Debug,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tracing::{info, instrument};

/// try to read from csv cache, else load directly from dir
#[instrument]
pub fn try_load_cache<P: AsRef<Path> + Debug>(path: P) -> Result<Vec<AudioTrack>, AppError> {
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }
    let mut rdr = app_reader()?;
    let mut tracks = rdr
        .records()
        .flat_map(|record| AudioTrack::from_record(record?))
        .collect::<Vec<AudioTrack>>();
    tracks.sort();

    info!("loaded {} items from cache", tracks.len());
    Ok(tracks)
}

fn record_hash(record: &StringRecord) -> Option<&str> {
    record.get(record.len() - 1)
}

/// handles collisions when there are multiple csv entries with the same hash
pub fn handle_collision() -> Result<(), AppError> {
    let mut rdr = app_reader()?;
    let mut records: Vec<StringRecord> = rdr.records().flatten().collect();

    records.sort_by(|a, b| record_hash(a).cmp(&record_hash(b)));
    records.dedup_by(|a, b| record_hash(a) == record_hash(b));

    let mut writer = app_writer_non_append()?;
    for rec in records {
        writer.write_record(&rec)?;
    }

    Ok(())
}

/// add hash to existing tracks inside the client
///
/// * `handle`: handle of the hashing runtime
/// * `also_write`: wheter if a write action should also be executed after
/// calculating the hash
#[instrument(skip(lib_arc, handle))]
pub async fn inject_hash(
    lib_arc: Arc<Mutex<LibraryClient>>,
    handle: Handle,
    also_write: bool,
) -> Result<(), AppError> {
    let tracks = lib_arc.clone().lock().map(|lib| lib.audio_tracks.clone())?;

    let mut join_set = JoinSet::new();

    for (index, track) in tracks.into_iter().enumerate() {
        let arced = lib_arc.clone();
        let _ = join_set.spawn_on(
            async move {
                let hash = hash_file(track.path.clone(), HashKind::XxHash)?;

                let track = {
                    // hash insertion
                    let mut guard = arced.lock().unwrap();
                    let track = guard
                        .audio_tracks
                        .get_mut(index)
                        .expect("index and size should stay unchanged");
                    track.binary_hash = Some(hash);
                    track.clone()
                };
                Ok::<AudioTrack, AppError>(track)
            },
            &handle,
        );
    }

    if also_write {
        let mut writer = app_writer_append()?;
        while let Some(Ok(audio)) = join_set.join_next().await {
            if also_write {
                let record = audio?.to_record();
                writer.write_record(&record)?;
                writer.flush()?;
            }
        }

        handle_collision()?;
    }

    // blocking on join_set in case `also_write` is false
    while let Some(t) = join_set.join_next().await {
        let _ = t.unwrap();
    }
    Ok(())
}
