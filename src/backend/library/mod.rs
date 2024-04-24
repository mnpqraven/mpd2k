use crate::{backend::utils::is_supported_audio, dotfile::DotfileSchema, error::AppError};
use audiotags::Tag;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
    _meta: DirEntry,
}

pub fn load_all_tracks(config: &DotfileSchema) -> Result<Vec<AudioTrack>, AppError> {
    let mut tracks: Vec<AudioTrack> = Vec::new();
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
            let tag = Tag::new().read_from_path(path.clone()).unwrap();
            let name = tag.title().unwrap_or(&filename).to_string();
            let album = tag.album_title().map(|e| e.to_owned());
            let artist = tag.artist().map(|e| e.to_owned());
            let album_artist = tag.album_artist().map(|e| e.to_owned());
            let track_no = tag.track_number();

            let track = AudioTrack {
                name: name.clone(),
                path,
                track_no,
                _meta: entry,
                artist,
                album,
                album_artist,
            };

            tracks.push(track)
        }
    }
    Ok(tracks)
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
