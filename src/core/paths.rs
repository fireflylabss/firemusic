use super::error::{Error, Result};
use std::path::{Component, Path};

const ALLOWED_URL_SCHEMES: &[&str] = &["http", "https"];

pub fn validate_url(url: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidInput("empty URL".to_string()));
    }

    let scheme = trimmed
        .split("://")
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !ALLOWED_URL_SCHEMES.contains(&scheme.as_str()) {
        return Err(Error::InvalidInput(format!(
            "unsupported URL scheme (only http/https allowed): {}",
            url
        )));
    }

    Ok(())
}

pub fn validate_playback_input(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidInput("empty playback input".to_string()));
    }

    if trimmed.contains('\0') {
        return Err(Error::InvalidInput("invalid playback input".to_string()));
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        validate_url(trimmed)?;
        return Ok(trimmed.to_string());
    }

    let path = Path::new(trimmed);
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::InvalidInput(format!(
            "local paths must not contain '..' components: {}",
            input
        )));
    }

    if !path.exists() {
        return Err(Error::InvalidInput(format!(
            "local file does not exist: {}",
            input
        )));
    }

    let canonical = path.canonicalize().map_err(Error::Io)?;
    if !canonical.is_file() {
        return Err(Error::InvalidInput(format!(
            "local playback input is not a file: {}",
            canonical.display()
        )));
    }

    Ok(canonical.to_string_lossy().into_owned())
}

pub fn validate_playback_inputs(inputs: &[String]) -> Result<Vec<String>> {
    if inputs.is_empty() {
        return Err(Error::InvalidInput(
            "at least one playback input is required".to_string(),
        ));
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