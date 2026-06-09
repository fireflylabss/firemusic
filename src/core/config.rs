use super::error::{Error, Result};
use std::path::{Component, PathBuf};

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("firemusic")
}

pub fn presets_dir() -> PathBuf {
    config_dir().join("presets")
}

pub fn playlists_dir() -> PathBuf {
    config_dir().join("playlists")
}

pub fn default_music_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Music")
}

pub fn resolve_music_dir(dir: &str) -> Result<PathBuf> {
    if dir.is_empty() {
        return Ok(default_music_dir());
    }

    let path = PathBuf::from(dir);
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::ConfigPath(
            "music directory must not contain '..' components".to_string(),
        ));
    }

    if path.exists() {
        let canonical = path.canonicalize().map_err(Error::Io)?;
        if !canonical.is_dir() {
            return Err(Error::ConfigPath(format!(
                "music directory is not a folder: {}",
                canonical.display()
            )));
        }
        return Ok(canonical);
    }

    if !path.is_absolute() {
        return Err(Error::ConfigPath(format!(
            "music directory must be an existing absolute path: {}",
            dir
        )));
    }

    Err(Error::ConfigPath(format!(
        "music directory does not exist: {}",
        dir
    )))
}