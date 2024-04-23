use crate::{backend::utils::is_supported_audio, dotfile::DotfileSchema, error::AppError};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    _meta: DirEntry,
}

pub fn load_all_tracks(config: &DotfileSchema) -> Result<Vec<AudioTrack>, AppError> {
    let mut tracks: Vec<AudioTrack> = Vec::new();
    let root = config.library_root()?;

    let library_tree = WalkDir::new(root).follow_links(true);

    for entry in library_tree {
        let entry = entry?;
        let (name, path) = (
            entry.file_name().to_string_lossy().to_string(),
            entry.path().to_string_lossy().to_string(),
        );

        if is_supported_audio(&path) {
            let track = AudioTrack {
                name,
                path,
                _meta: entry,
            };

            tracks.push(track)
        }
    }
    Ok(tracks)
}

#[cfg(test)]
mod tests {
    use crate::dotfile::DotfileSchema;

    use super::load_all_tracks;

    #[test]
    #[cfg(windows)]
    fn display() {
        let cfg = DotfileSchema::parse().unwrap();
        let tracks = load_all_tracks(&cfg);
        assert!(tracks.is_ok());
        assert!(!tracks.unwrap().is_empty());
    }
}
