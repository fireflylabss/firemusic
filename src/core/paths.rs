use anyhow::{bail, Result};
use std::path::{Component, Path, PathBuf};

const ALLOWED_URL_SCHEMES: &[&str] = &["http", "https"];

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("firemusic")
}

pub fn resolve_music_dir(dir: &str) -> Result<PathBuf> {
    if dir.is_empty() {
        return Ok(
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Music"),
        );
    }

    let path = PathBuf::from(dir);
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        bail!("music directory must not contain '..' components");
    }

    if path.exists() {
        let canonical = path.canonicalize()?;
        if !canonical.is_dir() {
            bail!("music directory is not a folder: {}", canonical.display());
        }
        return Ok(canonical);
    }

    if !path.is_absolute() {
        bail!("music directory must be an existing absolute path: {}", dir);
    }

    bail!("music directory does not exist: {}", dir)
}

pub fn validate_url(url: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        bail!("empty URL");
    }

    let scheme = trimmed
        .split("://")
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !ALLOWED_URL_SCHEMES.contains(&scheme.as_str()) {
        bail!("unsupported URL scheme (only http/https allowed): {}", url);
    }

    Ok(())
}

pub fn validate_playback_input(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("empty playback input");
    }

    if trimmed.contains('\0') {
        bail!("invalid playback input");
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        validate_url(trimmed)?;
        return Ok(trimmed.to_string());
    }

    let path = Path::new(trimmed);
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        bail!("local paths must not contain '..' components: {}", input);
    }

    if !path.exists() {
        bail!("local file does not exist: {}", input);
    }

    let canonical = path.canonicalize()?;
    if !canonical.is_file() {
        bail!("local playback input is not a file: {}", canonical.display());
    }

    Ok(canonical.to_string_lossy().into_owned())
}

pub fn validate_playback_inputs(inputs: &[String]) -> Result<Vec<String>> {
    if inputs.is_empty() {
        bail!("at least one playback input is required");
    }
    inputs.iter().map(|i| validate_playback_input(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_url_scheme() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("ftp://example.com/track.mp3").is_err());
    }

    #[test]
    fn accepts_http_and_https() {
        assert!(validate_url("https://example.com/track").is_ok());
        assert!(validate_url("http://example.com/track").is_ok());
    }
}