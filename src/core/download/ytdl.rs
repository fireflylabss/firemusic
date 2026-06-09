use anyhow::Result;
use std::process::Command;

pub(crate) fn video_format_filter(quality: &str) -> String {
    if quality == "best" {
        "bv+ba/b".to_string()
    } else {
        let height = quality.split('x').next_back().unwrap_or(quality);
        format!("bv[height<={height}]+ba/b[height<={height}]")
    }
}

pub fn run_ytdl_preset(url: &str, is_audio: bool) -> Result<()> {
    let mut cmd = Command::new("yt-dlp");
    if is_audio {
        println!("\u{1F525} downloading audio (mp3/best)...");
        cmd.args(["-x", "--audio-format", "mp3", "--audio-quality", "0", url]);
    } else {
        println!("\u{1F525} downloading video (mp4/best)...");
        cmd.args(["-f", "bv+ba/b", "--merge-output-format", "mp4", url]);
    }

    let status = cmd.spawn()?.wait()?;
    if status.success() {
        println!("\u{2705} download complete.");
    } else {
        println!("\u{274C} download failed.");
    }
    Ok(())
}

pub struct DownloadConfig {
    pub format: String,
    pub title: String,
    pub is_audio: bool,
    pub quality: Option<String>,
}

pub fn run_configured_download(url: &str, config: &DownloadConfig, common_args: &[String]) -> Result<()> {
    println!(
        "   \u{1F4E5} downloading [{}]: {}",
        config.format, config.title
    );
    let mut cmd = Command::new("yt-dlp");
    cmd.args(common_args);
    if config.is_audio {
        cmd.args([
            "-x",
            "--audio-format",
            &config.format,
            "--audio-quality",
            "0",
        ]);
    } else {
        let res_filter = video_format_filter(config.quality.as_deref().unwrap_or("best"));
        cmd.args(["-f", &res_filter, "--merge-output-format", &config.format]);
    }
    cmd.args(["-o", "%(title).200B [%(id)s].%(ext)s"]);
    cmd.arg(url);
    let status = cmd.spawn()?.wait()?;
    if !status.success() {
        println!("\u{274C} download failed for {}", url);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_format_filter_best() {
        assert_eq!(video_format_filter("best"), "bv+ba/b");
    }

    #[test]
    fn video_format_filter_resolution() {
        assert_eq!(
            video_format_filter("1920x1080"),
            "bv[height<=1080]+ba/b[height<=1080]"
        );
        assert_eq!(
            video_format_filter("1080"),
            "bv[height<=1080]+ba/b[height<=1080]"
        );
    }
}