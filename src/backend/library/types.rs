use crate::{backend::utils::empty_to_option, error::AppError};
use audiotags::TimestampTag;
use chrono::{Datelike, NaiveDate};
use csv::StringRecord;
use std::{
    cmp::Ordering,
    fmt::Display,
    fs::read_dir,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
#[derive(Debug, strum::Display)]
pub enum HashKind {
    Sha256,
    Murmur,
    XxHash,
}

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

#[derive(Debug, PartialEq, Eq)]
pub struct AlbumMeta {
    pub album_artist: Option<String>,
    pub date: SomeAlbumDate,
    pub name: Option<String>,
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

impl AudioTrack {
    const CSV_COLS: usize = 8;

    // TODO: perf + unicode check
    pub fn new<P: AsRef<Path> + ToString>(path: P) -> Self {
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

    pub fn to_record(&self) -> StringRecord {
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

    pub fn from_record(record: StringRecord) -> Result<Self, AppError> {
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
            date: SomeAlbumDate(AlbumDate::parse(TimestampTag::Unknown(
                record[6].to_string(),
            ))),
            binary_hash: empty_to_option(&record[7]),
        };

        Ok(track)
    }

    pub fn try_cover_path(&self) -> Option<PathBuf> {
        let track_path = PathBuf::from(self.path.clone());
        let dir = track_path.parent();
        if let Some(dir) = dir {
            let img_paths: Vec<PathBuf> = read_dir(dir)
                .unwrap()
                .filter(|e| {
                    let path = e.as_ref().unwrap().path();
                    ["png", "jpg"]
                        .into_iter()
                        .any(|ext| ext == path.extension().unwrap())
                })
                .map(|e| e.unwrap().path())
                .collect();
            return img_paths.first().cloned();
        }
        None
    }
}

impl AlbumDate {
    pub fn parse(text: TimestampTag) -> Option<Self> {
        match text {
            TimestampTag::Id3(_) => todo!(),
            TimestampTag::Unknown(text) =>
            // TODO: more formats
            {
                match NaiveDate::parse_from_str(&text, "%Y.%m.%d") {
                    Ok(s) => Some(Self {
                        year: s.year() as u32,
                        month: Some((s.month0() + 1) as u8),
                        day: Some((s.day0() + 1) as u8),
                    }),
                    Err(_) => None,
                }
            }
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

impl Ord for AlbumMeta {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.album_artist != other.album_artist {
            return self.album_artist.cmp(&other.album_artist);
        }
        if self.date != other.date {
            return self.date.cmp(&other.date);
        }
        if self.name != other.name {
            return self.name.cmp(&other.name);
        }
        Ordering::Equal
    }
}

impl PartialOrd for AlbumMeta {
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
