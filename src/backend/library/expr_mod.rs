use super::{hash::hash_file, AlbumMeta, AudioTrack, LibraryClient};
use crate::{backend::utils::is_supported_audio, client::PlayableClient, error::AppError};
use std::{
    fmt::Debug,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Handle, task::JoinSet};
use tracing::{error, instrument};
use walkdir::WalkDir;

#[instrument]
pub async fn load_all_tracks_expr<P: AsRef<Path> + Debug>(
    lib_root: P,
    lib_arc: Arc<Mutex<LibraryClient>>,
    handle: &Handle,
    // TODO: impl hard_update
) -> Result<(), AppError> {
    let library_tree = WalkDir::new(lib_root).follow_links(true);

    let mut tagging_task = JoinSet::new();

    for entry in library_tree {
        let entry = entry?;

        let path = entry.path().to_string_lossy().to_string();
        let lib_arc = lib_arc.clone();
        if is_supported_audio(&path) {
            tagging_task.spawn_on(
                async move {
                    let mut track = AudioTrack::new(path);
                    track.update_tag()?;

                    let track_meta = AlbumMeta::from(&track);

                    // add to album BTree
                    let mut lib = lib_arc.lock()?;
                    match lib.albums.get_mut(&track_meta) {
                        Some(val) => {
                            val.push(track.clone());
                            val.sort_unstable();
                        }
                        None => {
                            lib.albums.insert(track_meta, vec![track.clone()]);
                        }
                    }

                    Ok::<AudioTrack, AppError>(track)
                },
                handle,
            );
        }
    }

    while let Some(Ok(trk)) = tagging_task.join_next().await {
        if let Err(e) = trk {
            error!(?e);
        }
    }

    Ok(())
}

pub async fn load_hash_expr(
    lib_arc: Arc<Mutex<LibraryClient>>,
    handle: &Handle,
) -> Result<(), AppError> {
    let arc_outer = lib_arc.clone();
    let all: Vec<(AlbumMeta, Vec<Arc<str>>)> = arc_outer.lock().map(|lib| {
        lib.albums()
            .iter()
            .map(|(key, val)| {
                let paths: Vec<Arc<str>> = val
                    .iter()
                    .map(|e| e.path.clone())
                    .collect::<Vec<Arc<str>>>();
                (key.clone(), paths)
            })
            .collect()
    })?;

    let mut join_set = JoinSet::new();

    for (meta, paths) in all {
        let meta_arc = Arc::new(meta);

        for path in paths {
            let meta_arc_inner = meta_arc.clone();
            join_set.spawn_on(
                async move {
                    let hash = hash_file(path.as_ref(), super::HashKind::XxHash)?;
                    Ok::<(Arc<AlbumMeta>, Arc<str>, String), AppError>((meta_arc_inner, path, hash))
                },
                handle,
            );
        }
    }

    while let Some(Ok(t)) = join_set.join_next().await {
        let (meta, path, hash) = t?;
        let mut lib = lib_arc.lock()?;
        if let Some(tracks) = lib.albums.get_mut(&meta) {
            for track in tracks {
                if track.path == path {
                    track.binary_hash = Some(hash.clone().into());
                }
            }
        }
    }

    Ok(())
}

/// clean up duplicate tracks with the same hash
pub fn hash_cleanup(lib_arc: Arc<Mutex<LibraryClient>>) -> Result<(), AppError> {
    let mut lib = lib_arc.lock()?;
    for tracks in lib.albums.values_mut() {
        tracks.sort_unstable_by(|a, b| a.binary_hash.cmp(&b.binary_hash));
        tracks.dedup_by(|a, b| a.binary_hash == b.binary_hash);
        tracks.sort();
    }
    Ok(())
}
