use super::utils::empty_to_option;
use crate::{
    backend::utils::is_supported_audio,
    client::library::LibraryClient,
    error::{AppError, LibraryError},
};
use ::csv::StringRecord;
use audiotags::Tag;
use chrono::{Datelike, NaiveDate};
use core::cmp::Ordering;
use rodio::Decoder;
use std::{
    fmt::{Debug, Display},
    fs::File,
    io::BufReader,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Handle, task::JoinSet};
use tracing::instrument;
use walkdir::WalkDir;

pub mod cache;
mod csv;
pub mod hash;

// NOTE: keep expanding this or migrate to album(outer struct) > tracks(inner struct)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AudioTrack {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
    pub date: SomeAlbumDate,
    pub binary_hash: Option<String>,
}

impl AudioTrack {
    const CSV_COLS: usize = 8;

    // TODO: perf + unicode check
    fn new<P: AsRef<Path> + ToString>(path: P) -> Self {
        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        Self {
            name,
            path: path.to_string(),
            artist: None,
            album: None,
            album_artist: None,
            track_no: None,
            date: SomeAlbumDate(None),
            binary_hash: None,
        }
    }

    fn to_record(&self) -> StringRecord {
        let as_vec: &[String; Self::CSV_COLS] = &[
            self.name.clone(),
            self.path.clone(),
            self.track_no.map(|no| no.to_string()).unwrap_or_default(),
            self.artist.clone().unwrap_or_default(),
            self.album.clone().unwrap_or_default(),
            self.album_artist.clone().unwrap_or_default(),
            self.date
                .0
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default(),
            // ----
            // NOTE: ALWAYS PUT THIS LAST FOR `record_hash`
            self.binary_hash.clone().unwrap_or_default(),
        ];
        StringRecord::from(as_vec.as_slice())
    }

    fn from_record(record: StringRecord) -> Result<Self, AppError> {
        if record.len() != Self::CSV_COLS {
            return Err(AppError::CsvParse);
        }
        let track = AudioTrack {
            name: record[0].to_string(),
            path: record[1].to_string(),
            track_no: empty_to_option(&record[2]),
            artist: empty_to_option(&record[3]),
            album: empty_to_option(&record[4]),
            album_artist: empty_to_option(&record[5]),
            date: SomeAlbumDate(AlbumDate::parse(&record[6])),
            binary_hash: empty_to_option(&record[7]),
        };

        Ok(track)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SomeAlbumDate(pub Option<AlbumDate>);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct AlbumDate {
    // at year is always `Some`, if we can't parse year then the whole struct is safe to be `None`
    pub year: u32,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl AlbumDate {
    fn parse(text: &str) -> Option<Self> {
        // TODO: more formats
        match NaiveDate::parse_from_str(text, "%Y.%m.%d") {
            Ok(s) => Some(Self {
                year: s.year() as u32,
                month: Some((s.month0() + 1) as u8),
                day: Some((s.day0() + 1) as u8),
            }),
            Err(_) => None,
        }
    }
}

impl Display for AlbumDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.year.to_string();
        if let Some(month) = self.month {
            s.push_str(&format!(".{month}"));
        }
        if let Some(day) = self.day {
            s.push_str(&format!(".{day}"));
        }
        write!(f, "{}", s)
    }
}

impl Ord for AlbumDate {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.year != other.year {
            return self.year.cmp(&other.year);
        }
        if self.month != other.month {
            return self.month.cmp(&other.month);
        }
        self.day.cmp(&other.day)
    }
}
impl PartialOrd for AlbumDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SomeAlbumDate {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0, other.0) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}
impl PartialOrd for SomeAlbumDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AudioTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.album_artist != other.album_artist {
            return self.album_artist.cmp(&other.album_artist);
        }
        if self.date != other.date {
            return self.date.cmp(&other.date);
        }
        if self.album != other.album {
            // None album goes last
            return self.album.cmp(&other.album);
        }
        if self.track_no != other.track_no {
            return self.track_no.cmp(&other.track_no);
        }
        if self.path != other.path {
            return self.path.cmp(&other.path);
        }
        Ordering::Equal
    }
}

impl PartialOrd for AudioTrack {
    /// album artist > date > album name > disc no > track no > path
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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
