use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute, queue,
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    style::Stylize,
};
use dialoguer::Input;
use libmpv2::{events::Event as MEvent, Mpv};
use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

mod tactical_select;
use tactical_select::tactical_select;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum DownloadPreset {
    Audio,
    Video,
}

/// 🔥 fire music - high-performance minimalist cli music player.
#[derive(Parser, Debug)]
#[command(
    name = "firemusic",
    author = "FireflyLabs",
    version = "0.2.4",
    about = "🔥 fire music - minimalist high-performance cli music player",
    long_about = "fire music is a tactical cli player designed for pro users. \
                  it features a 'zero-leak' interface, advanced playlist logic, \
                  and an integrated multi-format download system via yt-dlp.",
    help_template = "\x1b[1m{name} v{version}\x1b[0m - {about}\n\n\
                     \x1b[1m{usage-heading}\x1b[0m {usage}\n\n\
                     \x1b[1mOPTIONS:\x1b[0m\n\
                     {options}\n\n\
                     \x1b[1mCONTROLS:\x1b[0m\n\
                       space                pause / resume\n\
                       ← / →, h / l         seek 5s\n\
                       {{ / }}                seek 1m\n\
                       ↑ / ↓, k / j         volume +/-\n\
                       1 - 9                jump to % (10-90%)\n\
                       0                    reset speed/pitch/eq\n\
                       + / -                speed +/-\n\
                       , / .                pitch down / up\n\
                       e                    cycle equalizer\n\
                       l                    toggle loop mode\n\
                       m                    toggle mute\n\
                       s                    back to search menu (at end)\n\
                       q, esc               quit session\n\n\
                     \x1b[1mDISCOVERY:\x1b[0m\n\
                       msc -s               open interactive hub (youtube, soundcloud, ytm)\n\
                       msc -s \"query\"       quick search on youtube\n\
                       msc -s \"sc:query\"    quick search on soundcloud\n\n\
                     \x1b[1mDOWNLOAD:\x1b[0m\n\
                       msc --download       start multi-format interactive wizard\n\
                       msc -d=audio \"url\"    fast high-quality mp3 download\n\
                       msc -d=video \"url\"    fast 1080p mp4 download\n\n\
                     \x1b[1mEXAMPLES:\x1b[0m\n\
                       msc song.mp3          play local file with defaults\n\
                       msc -s \"not like us\"  search and play across providers\n",
    disable_help_subcommand = true,
)]
struct Args {
    /// paths or urls (youtube, soundcloud, etc)
    #[arg(
        required_unless_present = "download", 
        required_unless_present = "search", 
        num_args = 0.., 
        value_name = "INPUT"
    )]
    inputs: Vec<String>,

    /// repeat track infinitely
    #[arg(short, long = "loop", alias = "loop-mode", help = "enable infinite loop mode")]
    loop_mode: bool,

    /// initial playback speed (0.01 - 100.0)
    #[arg(short = 'f', long, default_value_t = 1.0, value_name = "F", help = "set initial speed factor")]
    speed: f64,

    /// volume level (0 - 100)
    #[arg(short, long, default_value_t = 100.0, value_name = "L", help = "set initial volume level")]
    volume: f64,

    /// download media (experimental)
    #[arg(
        short, 
        long, 
        num_args = 0..=1, 
        value_name = "MODE", 
        require_equals = true, 
        default_missing_value = "interactive",
        help = "start download wizard or use a preset (audio/video)"
    )]
    download: Option<String>,

    /// search for music across platforms
    #[arg(
        short, 
        long, 
        num_args = 0..=1, 
        value_name = "QUERY", 
        require_equals = false, 
        default_missing_value = "",
        help = "search and play or download music interactively"
    )]
    search: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct SearchResult {
    title: String,
    url: Option<String>,
    webpage_url: Option<String>,
    id: Option<String>,
    duration: Option<f64>,
    uploader: Option<String>,
    channel: Option<String>,
    artist: Option<String>,
    subtitles: Option<serde_json::Value>,
    automatic_captions: Option<serde_json::Value>,
    #[serde(skip)]
    provider: String,
}

impl SearchResult {
    fn has_subs(&self) -> bool {
        self.subtitles.as_ref().map_or(false, |s| s.is_object() && !s.as_object().unwrap().is_empty()) ||
        self.automatic_captions.as_ref().map_or(false, |s| s.is_object() && !s.as_object().unwrap().is_empty())
    }

    fn get_playable_url(&self) -> String {
        if let Some(u) = &self.url {
            if u.starts_with("http") {
                return u.clone();
            }
        }
        if let Some(u) = &self.webpage_url {
            if u.starts_with("http") {
                return u.clone();
            }
        }
        if let Some(id) = &self.id {
            if self.provider == "yt" || self.provider == "ym" {
                return format!("https://www.youtube.com/watch?v={}", id);
            }
        }
        self.url.clone().unwrap_or_default()
    }

    fn get_uploader(&self) -> &str {
        if let Some(a) = &self.artist { return a; }
        if let Some(c) = &self.channel { return c; }
        if let Some(u) = &self.uploader { return u; }
        "?"
    }
}

#[derive(Deserialize, Debug)]
struct YtdlFormat {
    #[allow(dead_code)]
    format_id: String,
    #[allow(dead_code)]
    ext: String,
    resolution: Option<String>,
    vcodec: Option<String>,
}

#[derive(Deserialize, Debug)]
struct YtdlInfo {
    title: String,
    formats: Vec<YtdlFormat>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(mode) = args.download {
        return handle_download(&mode, args.inputs);
    }

    if let Some(query) = args.search {
        return handle_search(query, args.loop_mode, args.volume, args.speed);
    }

    let mut mpv = Mpv::new().map_err(|e| anyhow::anyhow!("mpv init error: {:?}", e))?;
    
    // Core Setup
    mpv.set_property("video", "no").ok();
    mpv.set_property("volume", args.volume).ok();
    mpv.set_property("speed", args.speed).ok();
    mpv.set_property("ytdl", "yes").ok();
    mpv.set_property("ytdl-format", "bestaudio/best").ok();
    
    let is_loop = args.loop_mode;
    if is_loop {
        mpv.set_property("loop-file", "inf").ok();
    }

    // Load sequential playlist
    for (i, input) in args.inputs.iter().enumerate() {
        let mode = if i == 0 { "replace" } else { "append" };
        mpv.command("loadfile", &[input, mode]).ok();
    }

    enable_raw_mode().context("terminal error")?;
    let mut stdout = io::stdout();
    println!("\n\n\n");
    execute!(stdout, cursor::Hide, cursor::MoveUp(3))?;

    let res = play_loop(&mut mpv, is_loop);
    
    execute!(stdout, cursor::Show, cursor::MoveToColumn(0), cursor::MoveDown(3))?;
    println!();
    disable_raw_mode().ok();
    
    if let Ok(PlayLoopResult::Quit) = res {
        println!("{}", "── fire music session closed ──".dark_grey());
    }
    Ok(())
}

struct SearchProvider {
    id: &'static str,
    label: &'static str,
    prefix: &'static str,
    suffix: &'static str,
}

const PROVIDERS: &[SearchProvider] = &[
    SearchProvider { id: "yt", label: "youtube", prefix: "ytsearch", suffix: "" },
    SearchProvider { id: "ym", label: "youtube-music", prefix: "https://music.youtube.com/search?q=", suffix: "#songs" },
    SearchProvider { id: "sc", label: "soundcloud", prefix: "scsearch", suffix: "" },
];

fn handle_search(mut query: String, loop_mode: bool, volume: f64, speed: f64) -> Result<()> {
    let mut active_providers;
    let mut current_page = 1;

    if query.is_empty() {
        query = Input::new()
            .with_prompt("🔍 search music")
            .interact_text()?;
            
        let p_labels: Vec<String> = PROVIDERS.iter().map(|p| p.label.to_string()).collect();
        if let Some(p_indices) = tactical_select("🌐 select a provider", &p_labels, false)? {
            if !p_indices.is_empty() {
                active_providers = vec![PROVIDERS[p_indices[0]].id.to_string()];
            } else {
                active_providers = vec!["yt".to_string()];
            }
        } else {
            return Ok(()); // user cancelled
        }
    } else if query.contains(':') {
        let (prefix, rest) = if let Some((p, r)) = query.split_once(':') {
            if PROVIDERS.iter().any(|prov| prov.id == p) {
                (vec![p.to_string()], r.to_string())
            } else {
                (vec!["yt".to_string()], query.clone())
            }
        } else {
            (vec!["yt".to_string()], query.clone())
        };
        query = rest;
        active_providers = prefix;
    } else {
        active_providers = vec!["yt".to_string()];
    }

    loop {
        let mut results = Vec::new();
        for pid in &active_providers {
            let provider = PROVIDERS.iter().find(|p| p.id == pid).unwrap();
            println!("🔍 searching {} for \"{}\" (page {})...", provider.label.cyan(), query.clone().white().bold(), current_page);
            
            let mut args = vec!["-j"];
            let items_range = format!("{}-{}", (current_page - 1) * 10 + 1, current_page * 10);
            args.extend(["--playlist-items", &items_range]);

            if provider.id == "yt" || provider.id == "sc" {
                args.push("--flat-playlist");
            }
            
            let query_str = if provider.id == "ym" {
                format!("{}{}{}", provider.prefix, query, provider.suffix)
            } else {
                format!("{}{}:{}", provider.prefix, 10 * current_page, query)
            };
            args.push(&query_str);

            let output = Command::new("yt-dlp")
                .args(&args)
                .output()?;

            if output.status.success() {
                let out_str = String::from_utf8_lossy(&output.stdout);
                for line in out_str.lines() {
                    match serde_json::from_str::<SearchResult>(line) {
                        Ok(mut res) => {
                            res.provider = pid.to_string();
                            results.push(res);
                        }
                        Err(_) => {}
                    }
                }
            } else {
                eprintln!("yt-dlp error for {}: {}", pid, String::from_utf8_lossy(&output.stderr));
            }
        }

        if results.is_empty() {
            println!("❌ no results found.");
            let action_items = vec!["new search".to_string(), "previous page".to_string(), "quit".to_string()];
            let action = match tactical_select("no results. what now?", &action_items, false)? {
                Some(s) if !s.is_empty() => s[0],
                _ => 2,
            };
            match action {
                0 => { query = Input::new().with_prompt("🔍 search music").interact_text()?; current_page = 1; continue; }
                1 => { if current_page > 1 { current_page -= 1; } continue; }
                _ => return Ok(()),
            }
        }

        let mut items: Vec<String> = results.iter()
            .map(|r| {
                let dur = r.duration.map(|d| format!("({:02}:{:02})", (d/60.) as i32, (d%60.) as i32)).unwrap_or_default();
                let cc = if r.has_subs() { " [CC]".green().to_string() } else { "".to_string() };
                format!("[{}] {} {}{} | {}", r.provider.clone().cyan(), r.title.clone().white(), dur.yellow(), cc, r.get_uploader().dark_grey())
            })
            .collect();

        items.push("──────────────────────────────────────────────────".dark_grey().to_string());
        items.push("[➔ next page]".to_string());
        if current_page > 1 { items.push("[➔ previous page]".to_string()); }
        items.push("[➔ change provider]".to_string());
        items.push("[➔ new search]".to_string());

        let selections = match tactical_select(&format!("🔥 results for \"{}\" (Page {})", query.clone(), current_page), &items, true)? {
            Some(s) => s,
            None => return Ok(()),
        };

        if selections.is_empty() { return Ok(()); }

        let mut final_urls = Vec::new();
        let mut switch_provider = false;
        let mut new_search = false;
        let mut next_page = false;
        let mut prev_page = false;

        for &idx in &selections {
            if idx < results.len() {
                let url = results[idx].get_playable_url();
                if !url.is_empty() { final_urls.push(url); }
            } else {
                let label = &items[idx];
                if label.contains("next page") { next_page = true; }
                else if label.contains("previous page") { prev_page = true; }
                else if label.contains("change provider") { switch_provider = true; }
                else if label.contains("new search") { new_search = true; }
            }
        }

        if next_page { current_page += 1; continue; }
        if prev_page { if current_page > 1 { current_page -= 1; } continue; }

        if switch_provider {
            let p_labels: Vec<String> = PROVIDERS.iter().map(|p| p.label.to_string()).collect();
            if let Some(p_indices) = tactical_select("🌐 select a provider", &p_labels, false)? {
                if !p_indices.is_empty() {
                    active_providers = vec![PROVIDERS[p_indices[0]].id.to_string()];
                    current_page = 1;
                }
            }
            continue;
        }

        if new_search {
            query = Input::new().with_prompt("🔍 search music").interact_text()?;
            current_page = 1;
            continue;
        }

        if final_urls.len() == 1 {
            let action_items = vec!["play now".to_string(), "select quality & play".to_string(), "download".to_string(), "back".to_string()];
            let action = match tactical_select(&format!("🎵 selected: {}", results[selections[0]].title), &action_items, false)? {
                Some(s) if !s.is_empty() => s[0],
                _ => continue,
            };

            match action {
                0 | 1 => {
                    let mut current_url = final_urls[0].clone();
                    let mut current_quality = "bestaudio/best".to_string();
                    
                    if action == 1 {
                        let q_options = vec!["high (best)".to_string(), "medium (128k)".to_string(), "low (data saving)".to_string()];
                        if let Some(q_idx) = tactical_select("🎧 select audio quality", &q_options, false)? {
                            current_quality = match q_idx[0] {
                                0 => "bestaudio/best".to_string(),
                                1 => "bestaudio[abr<=128]/best".to_string(),
                                2 => "worstaudio/worst".to_string(),
                                _ => "bestaudio/best".to_string(),
                            };
                        }
                    }

                    loop {
                        let mut mpv = Mpv::new().map_err(|e| anyhow::anyhow!("mpv init: {:?}", e))?;
                        mpv.set_property("video", "no").ok();
                        mpv.set_property("volume", volume).ok();
                        mpv.set_property("speed", speed).ok();
                        mpv.set_property("ytdl", "yes").ok();
                        mpv.set_property("ytdl-format", current_quality.as_str()).ok();
                        if loop_mode { mpv.set_property("loop-file", "inf").ok(); }
                        mpv.command("loadfile", &[&current_url, "replace"]).ok();
                        
                        enable_raw_mode().ok();
                        println!("\n\n\n");
                        execute!(io::stdout(), cursor::Hide, cursor::MoveUp(3))?;
                        let play_res = play_loop(&mut mpv, loop_mode)?;
                        execute!(io::stdout(), cursor::Show, cursor::MoveToColumn(0), cursor::MoveDown(3))?;
                        disable_raw_mode().ok();
                        
                        match play_res {
                            PlayLoopResult::SearchAgain => {
                                query = Input::new().with_prompt("🔍 search music").interact_text()?;
                                current_page = 1;
                                break;
                            },
                            PlayLoopResult::EndReached => {
                                let end_options = vec![
                                    "[➔ repeat this track]".to_string(),
                                    "[➔ new search]".to_string(),
                                    "[➔ back to results]".to_string(),
                                    "[➔ download this track]".to_string(),
                                    "[q] quit".to_string(),
                                ];
                                match tactical_select("🏁 track ended. what now?", &end_options, false)? {
                                    Some(s) if s[0] == 0 => continue,
                                    Some(s) if s[0] == 1 => {
                                        query = Input::new().with_prompt("🔍 search music").interact_text()?;
                                        current_page = 1;
                                        break;
                                    },
                                    Some(s) if s[0] == 2 => break,
                                    Some(s) if s[0] == 3 => {
                                        handle_download("interactive", vec![current_url.clone()])?;
                                        return Ok(());
                                    },
                                    _ => return Ok(()),
                                }
                            },
                            PlayLoopResult::Quit => return Ok(()),
                        }
                    }
                }
                1 => {
                    handle_download("interactive", final_urls)?;
                    return Ok(());
                },
                _ => continue,
            }
        } else {
            handle_download("interactive", final_urls)?;
            return Ok(());
        }
    }
}

fn handle_download(mode: &str, urls: Vec<String>) -> Result<()> {
    match mode {
        "audio" => {
            if urls.is_empty() { anyhow::bail!("url required for --download=audio"); }
            for url in urls { run_ytdl_preset(&url, true)?; }
            Ok(())
        }
        "video" => {
            if urls.is_empty() { anyhow::bail!("url required for --download=video"); }
            for url in urls { run_ytdl_preset(&url, false)?; }
            Ok(())
        }
        _ => run_interactive_download(urls),
    }
}

fn run_ytdl_preset(url: &str, is_audio: bool) -> Result<()> {
    let mut cmd = Command::new("yt-dlp");
    if is_audio {
        println!("🔥 downloading audio (mp3/best)...");
        cmd.args(["-x", "--audio-format", "mp3", "--audio-quality", "0", url]);
    } else {
        println!("🔥 downloading video (mp4/best)...");
        cmd.args(["-f", "bv+ba/b", "--merge-output-format", "mp4", url]);
    }
    
    let status = cmd.spawn()?.wait()?;
    if status.success() {
        println!("✅ download complete.");
    } else {
        println!("❌ download failed.");
    }
    Ok(())
}

struct DownloadConfig {
    format: String,
    filename: String,
    is_audio: bool,
    quality: Option<String>,
}

fn run_interactive_download(mut urls: Vec<String>) -> Result<()> {
    println!("{}", "🔥 fire music download wizard".bold().yellow());
    
    if urls.is_empty() {
        let url: String = Input::new()
            .with_prompt("link to download")
            .interact_text()?;
        urls.push(url);
    }

    // 1. Selection Phase: Type of download
    let type_options = vec!["audio only".to_string(), "video only".to_string(), "both (audio + video)".to_string()];
    let type_idx = match tactical_select("📦 what would you like to download?", &type_options, false)? {
        Some(s) if !s.is_empty() => s[0],
        _ => return Ok(()),
    };

    let mut audio_formats = Vec::new();
    let mut video_formats = Vec::new();
    let mut video_quality = "best".to_string();

    // 2. Format Selection
    if type_idx == 0 || type_idx == 2 {
        let formats = vec!["mp3".to_string(), "m4a".to_string(), "wav".to_string(), "flac".to_string(), "opus".to_string()];
        if let Some(selected) = tactical_select("🎵 select audio formats", &formats, true)? {
            audio_formats = selected.iter().map(|&i| formats[i].clone()).collect();
        }
    }

    if type_idx == 1 || type_idx == 2 {
        let formats = vec!["mp4".to_string(), "mkv".to_string(), "webm".to_string()];
        if let Some(selected) = tactical_select("🎬 select video containers", &formats, true)? {
            video_formats = selected.iter().map(|&i| formats[i].clone()).collect();
        }
        
        if urls.len() == 1 {
            print!("\r\x1b[K🔍 fetching resolutions...");
            io::stdout().flush().unwrap();
            let output = Command::new("yt-dlp").args(["-j", &urls[0]]).output()?;
            print!("\r\x1b[K");
            if let Ok(info) = serde_json::from_slice::<YtdlInfo>(&output.stdout) {
                let mut res_options: Vec<String> = info.formats.iter()
                    .filter(|f| f.vcodec.as_deref().unwrap_or("none") != "none")
                    .filter_map(|f| f.resolution.clone())
                    .collect();
                res_options.sort();
                res_options.dedup();
                res_options.push("best".to_string());
                if let Some(res_idx) = tactical_select("📺 select video quality", &res_options, false)? {
                    video_quality = res_options[res_idx[0]].clone();
                }
            }
        }
    }

    // 3. Metadata & Extras
    let mut meta_options = vec!["embed metadata".to_string(), "embed thumbnail".to_string()];
    
    // Check if subtitles exist
    let mut available_subs = Vec::new();
    if !urls.is_empty() {
        print!("\r\x1b[K🔍 checking for subtitles...");
        io::stdout().flush().unwrap();
        let sub_check = Command::new("yt-dlp").args(["--list-subs", "--flat-playlist", &urls[0]]).output()?;
        print!("\r\x1b[K");
        let sub_out = String::from_utf8_lossy(&sub_check.stdout);
        
        let mut in_subs = false;
        for line in sub_out.lines() {
            if line.contains("Language") && line.contains("Formats") { in_subs = true; continue; }
            if in_subs && !line.trim().is_empty() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    available_subs.push(format!("{} - {}", parts[0], parts[1]));
                }
            }
        }
    }

    let compatible_for_subs = audio_formats.iter().any(|f| f == "m4a") || !video_formats.is_empty();
    if !available_subs.is_empty() {
        meta_options.push("download subtitles".to_string());
    }

    let mut common_args = Vec::new();
    let mut download_subs = false;

    if let Some(meta_selected) = tactical_select("✨ extra features (space to toggle, enter to confirm)", &meta_options, true)? {
        for &idx in &meta_selected {
            match meta_options[idx].as_str() {
                "embed metadata" => common_args.push("--embed-metadata".to_string()),
                "embed thumbnail" => common_args.push("--embed-thumbnail".to_string()),
                "download subtitles" => download_subs = true,
                _ => {}
            }
        }
    }

    if download_subs {
        let sub_types = vec!["integrated (embedded in file)".to_string(), "separate file (.srt/.vtt)".to_string()];
        let prompt = if compatible_for_subs { "📝 how would you like the subtitles?" } else { "📝 format incompatible for embedding. save as separate file?" };
        
        if let Some(choice) = tactical_select(prompt, &sub_types, false)? {
            let method = choice[0];
            
            if let Some(selected_subs) = tactical_select("🌍 select subtitle languages", &available_subs, true)? {
                let langs: Vec<String> = selected_subs.iter().map(|&i| available_subs[i].split(" - ").next().unwrap().to_string()).collect();
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

    // 4. Execution Phase
    for url in urls {
        println!("\n🚀 processing: {}", url.clone().cyan());
        let output = Command::new("yt-dlp").args(["-j", &url]).output()?;
        let info = match serde_json::from_slice::<YtdlInfo>(&output.stdout) {
            Ok(i) => i,
            Err(_) => { println!("❌ failed to fetch metadata for {}", url); continue; }
        };

        let mut configs = Vec::new();
        for fmt in &audio_formats {
            configs.push(DownloadConfig { format: fmt.clone(), filename: info.title.clone(), is_audio: true, quality: None });
        }
        for fmt in &video_formats {
            configs.push(DownloadConfig { format: fmt.clone(), filename: info.title.clone(), is_audio: false, quality: Some(video_quality.clone()) });
        }

        for config in configs {
            println!("   📥 downloading [{}]: {}", config.format.clone().cyan(), config.filename.clone().white().bold());
            let mut cmd = Command::new("yt-dlp");
            cmd.args(&common_args);
            if config.is_audio {
                cmd.args(["-x", "--audio-format", &config.format, "--audio-quality", "0"]);
            } else {
                let res_filter = if config.quality.as_deref() == Some("best") { "bv+ba/b".to_string() } 
                                else { format!("bv[height<={pos}]+ba/b[height<={pos}]", pos = config.quality.unwrap_or_else(|| "1080".to_string()).split('x').last().unwrap_or("1080")) };
                cmd.args(["-f", &res_filter, "--merge-output-format", &config.format]);
            }
            cmd.args(["-o", &format!("{}.%(ext)s", config.filename)]);
            cmd.arg(&url);
            cmd.spawn()?.wait()?;
        }
    }
    println!("\n✅ all operations complete.");
    Ok(())
}

fn render_ui(mpv: &Mpv, is_loop: bool) -> Result<()> {
    let mut stdout = io::stdout();
    let (width, _) = terminal::size().unwrap_or((80, 24));
    let width = width as usize;

    let time = mpv.get_property::<f64>("time-pos").unwrap_or(0.0);
    let duration = mpv.get_property::<f64>("duration").unwrap_or(0.0);
    let paused = mpv.get_property::<bool>("pause").unwrap_or(false);
    let mute = mpv.get_property::<bool>("mute").unwrap_or(false);
    let volume = mpv.get_property::<f64>("volume").unwrap_or(0.0);
    let speed = mpv.get_property::<f64>("speed").unwrap_or(1.0);
    let pitch = mpv.get_property::<f64>("pitch").unwrap_or(1.0);
    let title = mpv.get_property::<String>("media-title").unwrap_or_else(|_| "...".to_string());
    let bitrate = mpv.get_property::<f64>("audio-bitrate").unwrap_or(0.0) / 1000.0;
    
    let af = mpv.get_property::<String>("af").unwrap_or_default();
    let eq_label = if af.contains("bass") && af.contains("treble") { "rock" }
                  else if af.contains("bass") { "bass+" }
                  else if af.contains("treble") { "treble+" }
                  else if af.contains("1000") { "vocal" }
                  else if af.contains("300") { "lofi" }
                  else { "off" };

    let status_base = if paused { "⏸" } else { "▶" };
    let status_str = if mute { format!("{} (mute)", status_base) } else { status_base.to_string() };
    let loop_tag = if is_loop { " · loop" } else { "" };
    let tech_tags = format!("  ·  {:.0}kbps{}", bitrate, loop_tag);

    queue!(stdout, cursor::MoveToColumn(0))?;
    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let status_styled = format!("{} ", status_str);
    let available_for_title = width.saturating_sub(status_styled.chars().count() + tech_tags.chars().count() + 2);
    let display_title = if title.chars().count() > available_for_title { format!("{}...", &title.chars().take(available_for_title.saturating_sub(3)).collect::<String>()) } else { title };
    print!("{}{}{}", status_styled.dark_red().bold(), display_title.white().bold(), tech_tags.dark_grey());
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let progress = if duration > 0.0 { time / duration } else { 0.0 };
    let time_str = format!("{:02}:{:02} / {:02}:{:02}", (time/60.) as i32, (time%60.) as i32, (duration/60.) as i32, (duration%60.) as i32);
    let specs_str = format!("  ·  vol {:>3.0}%  ·  spd {:.1}x  ·  ptch {:.1}x  ·  eq {}", volume, speed, pitch, eq_label);
    let bar_width = width.saturating_sub(time_str.len() + specs_str.len() + 4).max(10);
    let filled = (progress * bar_width as f64) as usize;
    print!("{} | ", time_str.dark_grey());
    print!("{}", "━".repeat(filled.saturating_sub(1)).white());
    if filled > 0 { print!("{}", "█".dark_red()); } else { print!("{}", "━".white()); }
    print!("{}", "─".repeat(bar_width.saturating_sub(filled)).dark_grey());
    print!("{}", specs_str.dark_grey());
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    
    let is_at_end = duration > 0.0 && time >= duration - 0.5;
    let show_s = is_at_end && !is_loop;
    
    let shortcuts = if show_s {
        "[space] pause | [←/→] seek | [↑/↓] vol | [+/-] spd | [,/.] pitch | [e] eq | [s] search | [q] quit"
    } else {
        "[space] pause | [←/→] seek | [↑/↓] vol | [+/-] spd | [,/.] pitch | [e] eq | [q] quit"
    };
    
    let pad = (width.saturating_sub(shortcuts.len()) / 2) as u16;
    queue!(stdout, cursor::MoveToColumn(pad))?;
    print!("{}", shortcuts.dark_grey());
    
    queue!(stdout, cursor::MoveToColumn(0), cursor::MoveUp(2))?;
    stdout.flush()?;
    Ok(())
}

enum PlayLoopResult {
    Quit,
    SearchAgain,
    EndReached,
}

fn play_loop(mpv: &mut Mpv, mut is_loop: bool) -> Result<PlayLoopResult> {
    loop {
        if let Some(event_result) = mpv.wait_event(0.0) {
            match event_result.map_err(|e| anyhow::anyhow!("mpv error: {:?}", e))? {
                MEvent::EndFile(_reason) => if !is_loop { 
                    let remaining: i64 = mpv.get_property("playlist-count").unwrap_or(0);
                    let pos: i64 = mpv.get_property("playlist-pos").unwrap_or(0);
                    if pos + 1 >= remaining || pos < 0 { 
                        return Ok(PlayLoopResult::EndReached);
                    }
                },
                MEvent::Shutdown => break,
                _ => {}
            }
        }
        if event::poll(Duration::from_millis(0))? {
            if let CEvent::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { break; }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('s') => {
                        let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
                        let duration: f64 = mpv.get_property("duration").unwrap_or(0.0);
                        let is_at_end = duration > 0.0 && time >= duration - 1.0;
                        if is_at_end && !is_loop {
                            return Ok(PlayLoopResult::SearchAgain);
                        }
                    }
                    KeyCode::Char('e') => {
                        let af: String = mpv.get_property("af").unwrap_or_default();
                        if af.is_empty() {
                            mpv.set_property("af", "bass=g=10").ok(); // Bass+
                        } else if af.contains("bass") && !af.contains("treble") {
                            mpv.set_property("af", "treble=g=10").ok(); // Treble+
                        } else if af.contains("treble") && !af.contains("bass") {
                            mpv.set_property("af", "bass=g=10,treble=g=10").ok(); // Rock
                        } else if af.contains("bass") && af.contains("treble") {
                            mpv.set_property("af", "equalizer=f=1000:width_type=h:width=200:g=10").ok(); // Vocal
                        } else if af.contains("1000") {
                            mpv.set_property("af", "equalizer=f=300:width_type=h:width=200:g=-10,equalizer=f=3000:width_type=h:width=200:g=-10").ok(); // Lofi
                        } else {
                            mpv.set_property("af", "").ok(); // Off
                        }
                    }
                    KeyCode::Char(',') => {
                        let cur: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                        mpv.set_property("pitch", (cur - 0.05).max(0.5)).ok();
                    }
                    KeyCode::Char('.') => {
                        let cur: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                        mpv.set_property("pitch", (cur + 0.05).min(2.0)).ok();
                    }
                    KeyCode::Char('{') => { mpv.command("seek", &["-60", "relative"]).ok(); }
                    KeyCode::Char('}') => { mpv.command("seek", &["60", "relative"]).ok(); }
                    KeyCode::Char(' ') => {
                        let p: bool = mpv.get_property("pause").unwrap_or(false);
                        mpv.set_property("pause", !p).ok();
                    }
                    KeyCode::Char('m') => {
                        let m: bool = mpv.get_property("mute").unwrap_or(false);
                        mpv.set_property("mute", !m).ok();
                    }
                    KeyCode::Char('l') => {
                        is_loop = !is_loop;
                        mpv.set_property("loop-file", if is_loop { "inf" } else { "no" }).ok();
                    }
                    KeyCode::Char('0') => { 
                        mpv.set_property("speed", 1.0).ok(); 
                        mpv.set_property("pitch", 1.0).ok();
                    }
                    KeyCode::Right => { mpv.command("seek", &["5", "relative"]).ok(); }
                    KeyCode::Left | KeyCode::Char('h') => { mpv.command("seek", &["-5", "relative"]).ok(); }
                    KeyCode::Up | KeyCode::Char('k') => { 
                        let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                        mpv.set_property("volume", (v + 5.0).min(100.0)).ok();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                        mpv.set_property("volume", (v - 5.0).max(0.0)).ok();
                    }
                    KeyCode::Char(c) if c.is_digit(10) && c != '0' => {
                        let pct = c.to_digit(10).unwrap() * 10;
                        mpv.command("seek", &[&pct.to_string(), "absolute-percent"]).ok();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                        mpv.set_property("speed", (s + 0.1).min(10.0)).ok();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                        mpv.set_property("speed", (s - 0.1).max(0.1)).ok();
                    }
                    _ => {}
                }
            }
        }
        render_ui(mpv, is_loop)?;
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(PlayLoopResult::Quit)
}
