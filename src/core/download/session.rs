use anyhow::Result;
use crossterm::style::Stylize;
use dialoguer::Input;
use std::io::{self, Write};
use std::process::Command;

use super::ytdl::{run_configured_download, DownloadConfig};
use crate::core::discovery::YtdlInfo;
use crate::core::paths::validate_url;
use crate::core::tactical_select::tactical_select;

pub fn run_interactive_download(mut urls: Vec<String>) -> Result<()> {
    println!("{}", "\u{1F525} fire music download wizard".bold().yellow());

    if urls.is_empty() {
        let url: String = Input::new()
            .with_prompt("link to download")
            .interact_text()?;
        validate_url(&url)?;
        urls.push(url.trim().to_string());
    }

    let type_options = vec![
        "audio only".to_string(),
        "video only".to_string(),
        "both (audio + video)".to_string(),
    ];
    let type_idx = match tactical_select(
        "\u{1F4E6} what would you like to download?",
        &type_options,
        false,
    )? {
        Some(s) if !s.is_empty() => s[0],
        _ => return Ok(()),
    };

    let mut audio_formats = Vec::new();
    let mut video_formats = Vec::new();
    let mut video_quality = "best".to_string();

    if type_idx == 0 || type_idx == 2 {
        let formats = vec![
            "mp3".to_string(),
            "m4a".to_string(),
            "wav".to_string(),
            "flac".to_string(),
            "opus".to_string(),
        ];
        if let Some(selected) = tactical_select("\u{1F3B5} select audio formats", &formats, true)? {
            audio_formats = selected.iter().map(|&i| formats[i].clone()).collect();
        }
    }

    if type_idx == 1 || type_idx == 2 {
        let formats = vec!["mp4".to_string(), "mkv".to_string(), "webm".to_string()];
        if let Some(selected) =
            tactical_select("\u{1F3AC} select video containers", &formats, true)?
        {
            video_formats = selected.iter().map(|&i| formats[i].clone()).collect();
        }

        if urls.len() == 1 {
            print!("\r\x1b[K\u{1F50D} fetching resolutions...");
            io::stdout().flush().unwrap();
            let output = Command::new("yt-dlp").args(["-j", &urls[0]]).output()?;
            print!("\r\x1b[K");
            if let Ok(info) = serde_json::from_slice::<YtdlInfo>(&output.stdout) {
                let mut res_options: Vec<String> = info
                    .formats
                    .iter()
                    .filter(|f| f.vcodec.as_deref().unwrap_or("none") != "none")
                    .filter_map(|f| f.resolution.clone())
                    .collect();
                res_options.sort();
                res_options.dedup();
                res_options.push("best".to_string());
                if let Some(res_idx) =
                    tactical_select("\u{1F4FA} select video quality", &res_options, false)?
                {
                    video_quality = res_options[res_idx[0]].clone();
                }
            }
        }
    }

    let meta_options = vec![
        "embed metadata".to_string(),
        "embed thumbnail".to_string(),
        "check subtitles".to_string(),
    ];

    let compatible_for_subs = audio_formats.iter().any(|f| f == "m4a") || !video_formats.is_empty();

    let mut common_args = Vec::new();
    let mut check_subs = false;

    if let Some(meta_selected) = tactical_select(
        "\u{2728} extra features (space to toggle, enter to confirm)",
        &meta_options,
        true,
    )? {
        for &idx in &meta_selected {
            match meta_options[idx].as_str() {
                "embed metadata" => common_args.push("--embed-metadata".to_string()),
                "embed thumbnail" => common_args.push("--embed-thumbnail".to_string()),
                "check subtitles" => check_subs = true,
                _ => {}
            }
        }
    }

    if check_subs {
        let mut available_subs = Vec::new();
        if !urls.is_empty() {
            print!("\r\x1b[K\u{1F50D} checking for subtitles...");
            io::stdout().flush().unwrap();
            let sub_check = Command::new("yt-dlp")
                .args(["--list-subs", "--flat-playlist", &urls[0]])
                .output()?;
            print!("\r\x1b[K");
            let sub_out = String::from_utf8_lossy(&sub_check.stdout);

            let mut in_subs = false;
            for line in sub_out.lines() {
                if line.contains("Language") && line.contains("Formats") {
                    in_subs = true;
                    continue;
                }
                if in_subs && !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        available_subs.push(format!("{} - {}", parts[0], parts[1]));
                    }
                }
            }
        }

        if available_subs.is_empty() {
            println!("\u{2139} no subtitles available for this media.");
        } else {
            let sub_types = vec![
                "integrated (embedded in file)".to_string(),
                "separate file (.srt/.vtt)".to_string(),
            ];
            let prompt = if compatible_for_subs {
                "\u{1F4DD} how would you like the subtitles?"
            } else {
                "\u{1F4DD} format incompatible for embedding. save as separate file?"
            };

            if let Some(choice) = tactical_select(prompt, &sub_types, false)? {
                let method = choice[0];

                if let Some(selected_subs) =
                    tactical_select("\u{1F30D} select subtitle languages", &available_subs, true)?
                {
                    let langs: Vec<String> = selected_subs
                        .iter()
                        .map(|&i| available_subs[i].split(" - ").next().unwrap().to_string())
                        .collect();
                    common_args.push("--sub-langs".to_string());
                    common_args.push(langs.join(","));

                    if method == 0 && compatible_for_subs {
                        common_args.push("--embed-subs".to_string());
                    } else {
                        common_args.push("--write-subs".to_string());
                        common_args.push("--write-auto-subs".to_string());
                    }
                }
            }
        }
    }

    for url in urls {
        println!("\n\u{1F680} processing: {}", url.clone().cyan());
        let output = Command::new("yt-dlp").args(["-j", &url]).output()?;
        if !output.status.success() {
            println!(
                "\u{274C} failed to fetch metadata for {}\n{}",
                url,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            continue;
        }
        let info = match serde_json::from_slice::<YtdlInfo>(&output.stdout) {
            Ok(i) => i,
            Err(err) => {
                println!("\u{274C} failed to parse metadata for {}: {}", url, err);
                continue;
            }
        };

        let mut configs = Vec::new();
        for fmt in &audio_formats {
            configs.push(DownloadConfig {
                format: fmt.clone(),
                title: info.title.clone(),
                is_audio: true,
                quality: None,
            });
        }
        for fmt in &video_formats {
            configs.push(DownloadConfig {
                format: fmt.clone(),
                title: info.title.clone(),
                is_audio: false,
                quality: Some(video_quality.clone()),
            });
        }

        for config in configs {
            run_configured_download(&url, &config, &common_args)?;
        }
    }
    println!("\n\u{2705} all operations complete.");
    Ok(())
}