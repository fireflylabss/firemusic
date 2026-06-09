use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::core::audio::eq::EqPreset;
use crate::core::config::{playlists_dir, presets_dir};

pub(crate) fn list_stem_names(dir: &Path, suffix: &str) -> Vec<String> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(suffix) {
                    names.push(name.trim_end_matches(suffix).to_string());
                }
            }
        }
    }
    names.sort();
    names
}

pub fn list_playlists() -> Vec<String> {
    list_stem_names(&playlists_dir(), ".m3u")
}

pub fn load_playlist_paths(name: &str) -> Result<Vec<String>> {
    let path = playlists_dir().join(format!("{}.m3u", name));
    let content = fs::read_to_string(&path)?;
    Ok(content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(str::to_string)
        .collect())
}

pub fn save_playlist_paths(name: &str, paths: &[impl AsRef<str>]) -> Result<()> {
    let dir = playlists_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.m3u", name));
    let content = paths
        .iter()
        .map(|p| format!("{}\n", p.as_ref()))
        .collect::<String>();
    fs::write(&path, content)?;
    Ok(())
}

pub fn delete_playlist(name: &str) -> Result<()> {
    fs::remove_file(playlists_dir().join(format!("{}.m3u", name)))?;
    Ok(())
}

pub fn list_eq_presets() -> Vec<String> {
    list_stem_names(&presets_dir(), ".json")
}

pub fn save_eq_preset(preset: &EqPreset) -> Result<()> {
    let dir = presets_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", preset.name));
    let json = serde_json::to_string_pretty(preset)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn load_eq_preset(name: &str) -> Result<EqPreset> {
    let path = presets_dir().join(format!("{}.json", name));
    let json = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&json)?)
}

pub fn delete_eq_preset(name: &str) -> Result<()> {
    fs::remove_file(presets_dir().join(format!("{}.json", name)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("firemusic-store-{label}-{nanos}"))
    }

    #[test]
    fn list_stem_names_filters_and_sorts() {
        let dir = temp_dir("list");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("beta.m3u"), "").unwrap();
        fs::write(dir.join("alpha.m3u"), "").unwrap();
        fs::write(dir.join("ignore.txt"), "").unwrap();

        assert_eq!(
            list_stem_names(&dir, ".m3u"),
            vec!["alpha".to_string(), "beta".to_string()]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn playlist_roundtrip_in_temp_dir() {
        let dir = temp_dir("playlist");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("favorites.m3u");
        fs::write(&path, "/music/a.flac\n# comment\n\n/music/b.mp3\n").unwrap();

        let loaded: Vec<String> = fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(str::to_string)
            .collect();
        assert_eq!(loaded, vec!["/music/a.flac", "/music/b.mp3"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn eq_preset_json_roundtrip() {
        let preset = EqPreset {
            name: "warm".to_string(),
            gains: [1.0, 0.5, 0.0, -0.5, 0.0, 0.0, 0.5, 1.0, 0.5, 0.0],
        };
        let json = serde_json::to_string_pretty(&preset).unwrap();
        let decoded: EqPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "warm");
        assert_eq!(decoded.gains, preset.gains);
    }
}