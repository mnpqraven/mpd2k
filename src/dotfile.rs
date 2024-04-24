#![allow(dead_code)]

use crate::error::AppError;
use dirs::config_dir;
use serde::Deserialize;
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct DotfileSchema {
    /// only linux supports use_mpd for now, setting this to `true` on windows
    /// would hard-crash the program
    general: GeneralSubSchema,
    library: Option<LibrarySubSchema>,
    mpd: Option<MpdSubSchema>,
}

#[derive(Debug, Deserialize)]
struct LibrarySubSchema {
    root: String,
}

#[derive(Debug, Deserialize)]
struct GeneralSubSchema {
    use_mpd: bool,
}

#[derive(Debug, Deserialize)]
struct MpdSubSchema {
    // better valid type that impls TCP addr
    addr: String,
    port: u16,
}

impl DotfileSchema {
    pub fn config_file_path() -> Result<PathBuf, AppError> {
        config_dir()
            .map(|path| path.join("mpd2k/config.toml"))
            .ok_or(AppError::NoConfig)
    }

    pub fn config_dir_path() -> Result<PathBuf, AppError> {
        config_dir()
            .map(|path| path.join("mpd2k"))
            .ok_or(AppError::NoConfig)
    }

    pub fn cache_path() -> Result<PathBuf, AppError> {
        config_dir()
            .map(|path| path.join("mpd2k/cache.csv"))
            .ok_or(AppError::NoConfig)
    }

    pub fn parse() -> Result<Self, AppError> {
        let dotfile_path = Self::config_file_path()?;
        let conf_str = read_to_string(dotfile_path).map_err(|_| AppError::NoConfig)?;
        let cfg = toml::from_str(&conf_str).map_err(|_| AppError::BadConfig)?;
        Ok(cfg)
    }

    pub fn library_root(&self) -> Result<PathBuf, AppError> {
        match &self.library {
            // TODO: path validation
            Some(library) => Ok(PathBuf::from(&library.root)),
            None => Err(AppError::BadConfig),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DotfileSchema;
    use crate::error::AppError;

    #[test]
    #[cfg(not(windows))]
    fn test_config_paths_linux() -> Result<(), AppError> {
        let config_dir = DotfileSchema::config_dir_path()?
            .to_string_lossy()
            .to_string();
        let config_file = DotfileSchema::config_file_path()?
            .to_string_lossy()
            .to_string();

        assert_eq!(config_dir, "/home/othi/.config/mpd2k");
        assert_eq!(config_file, "/home/othi/.config/mpd2k/config.toml");

        Ok(())
    }

    #[test]
    #[cfg(windows)]
    fn test_config_paths_linux() -> Result<(), AppError> {
        let config_dir = DotfileSchema::config_dir_path()?
            .to_string_lossy()
            .to_string();
        let config_file = DotfileSchema::config_file_path()?
            .to_string_lossy()
            .to_string();

        assert_eq!(config_dir, "C\\Users\\mnpqr\\AppData\\Roaming\\mpd2k");
        assert_eq!(
            config_dir,
            "C\\Users\\mnpqr\\AppData\\Roaming\\mpd2k\\config.toml"
        );

        Ok(())
    }
}
