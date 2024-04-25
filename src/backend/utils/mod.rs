#![allow(clippy::upper_case_acronyms)]

use std::{path::Path, str::FromStr};
use strum::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq, Eq)]
enum SupportedAudio {
    #[strum(ascii_case_insensitive, to_string = "mp3")]
    MP3,
    // MP4,
    #[strum(ascii_case_insensitive)]
    WAV,
    #[strum(ascii_case_insensitive)]
    VORBIS,
    #[strum(ascii_case_insensitive)]
    FLAC,
    #[strum(ascii_case_insensitive)]
    AAC,
}

/// Returns true if the file is a valid audio file with supported codec
pub fn is_supported_audio<T: AsRef<Path>>(path: T) -> bool {
    let path = path.as_ref();
    match path.extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy();
            SupportedAudio::from_str(&ext).is_ok()
        }
        None => false,
    }
}

/// this function converts empty string to None
pub fn empty_to_option<T: std::str::FromStr + std::default::Default>(text: &str) -> Option<T> {
    match text.is_empty() {
        true => None,
        false => Some(text.parse::<T>().unwrap_or_default()),
    }
}
