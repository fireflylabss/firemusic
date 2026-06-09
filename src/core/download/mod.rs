mod session;
mod ytdl;

use anyhow::Result;
use clap::ValueEnum;

use super::paths::validate_url;

pub use ytdl::run_ytdl_preset;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum DownloadPreset {
    Audio,
    Video,
}

pub fn handle_download(mode: &str, urls: Vec<String>) -> Result<()> {
    let validated_urls: Vec<String> = urls
        .into_iter()
        .map(|url| {
            validate_url(&url)?;
            Ok(url.trim().to_string())
        })
        .collect::<Result<_>>()?;

    match mode {
        "audio" => {
            if validated_urls.is_empty() {
                anyhow::bail!("url required for --download=audio");
            }
            for url in validated_urls {
                run_ytdl_preset(&url, true)?;
            }
            Ok(())
        }
        "video" => {
            if validated_urls.is_empty() {
                anyhow::bail!("url required for --download=video");
            }
            for url in validated_urls {
                run_ytdl_preset(&url, false)?;
            }
            Ok(())
        }
        _ => session::run_interactive_download(validated_urls),
    }
}