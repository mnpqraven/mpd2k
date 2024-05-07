use super::types::*;
use std::{cmp::Ordering, fmt::Display};

impl From<AudioTrack> for AlbumMeta {
    fn from(value: AudioTrack) -> Self {
        Self {
            album_artist: value.album_artist,
            date: value.date,
            name: value.album,
        }
    }
}

impl From<&AudioTrack> for AlbumMeta {
    fn from(value: &AudioTrack) -> Self {
        let value = value.to_owned();
        AlbumMeta::from(value)
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
