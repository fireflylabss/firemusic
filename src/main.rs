use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute, queue,
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    style::Stylize,
};
use dialoguer::{Input, MultiSelect, Select};
use libmpv2::{events::Event as MEvent, Mpv};
use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

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
    help_template = "{about-section}\n\n\
                     {usage-heading} {usage}\n\n\
                     OPTIONS:\n\
                     {options}\n\n\
                     CONTROLS:\n\
                       space                  pause / resume\n\
                       ← / →                  seek 5s (also [h / l])\n\
                       [ / ]                  seek 1m\n\
                       ↑ / ↓                  volume +/- (also [k / j])\n\
                       1 - 9                  jump to % (10% - 90%)\n\
                       + / -                  speed +/- ([0] to reset)\n\
                       l                      toggle loop mode\n\
                       m                      toggle mute\n\
                       q                      quit session\n\n\
                     EXAMPLES:\n\
                       msc song.mp3           play song with default settings\n\
                       msc track1.mp3 -l      play song in infinite loop\n\
                       msc --download         start interactive download wizard\n\
                       msc -d=audio \"url\"     quick download high-quality mp3\n",
    disable_help_subcommand = true,
)]
struct Args {
    /// paths or urls (youtube, soundcloud, etc)
    #[arg(required_unless_present = "download", num_args = 0.., value_name = "INPUT")]
    inputs: Vec<String>,

    /// repeat track infinitely
    #[arg(short, long = "loop", alias = "loop-mode", help = "enable infinite loop mode")]
    loop_mode: bool,

    /// initial playback speed (0.01 - 100.0)
    #[arg(short, long, default_value_t = 1.0, value_name = "F", help = "set initial speed factor")]
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

    /// start with lyrics enabled
    #[arg(short, long = "lyrics", help = "show lyrics by default if available")]
    show_lyrics: bool,
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
        return handle_download(&mode, args.inputs.first());
    }

    let mut mpv = Mpv::new().map_err(|e| anyhow::anyhow!("mpv init error: {:?}", e))?;
    
    // Core Setup
    mpv.set_property("video", "no").ok();
    mpv.set_property("audio-display", "no").ok();
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

    let res = play_loop(&mut mpv, is_loop, args.show_lyrics);
    
    execute!(stdout, cursor::Show, cursor::MoveToColumn(0), cursor::MoveDown(3))?;
    println!();
    disable_raw_mode().ok();
    
    if res.is_ok() {
        println!("{}", "── fire music session closed ──".dark_grey());
    }
    res
}

fn handle_download(mode: &str, input: Option<&String>) -> Result<()> {
    match mode {
        "audio" => {
            let url = input.context("URL required for --download=audio")?;
            run_ytdl_preset(url, true)
        }
        "video" => {
            let url = input.context("URL required for --download=video")?;
            run_ytdl_preset(url, false)
        }
        _ => run_interactive_download(),
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

fn run_interactive_download() -> Result<()> {
    println!("{}", "🔥 fire music download wizard (ultrawork experimental)".bold().yellow());
    
    let url: String = Input::new()
        .with_prompt("link to download")
        .interact_text()?;

    println!("🔍 fetching metadata & formats...");
    let output = Command::new("yt-dlp")
        .args(["-j", &url])
        .output()
        .context("failed to execute yt-dlp. is it installed?")?;

    if !output.status.success() {
        anyhow::bail!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let info: YtdlInfo = serde_json::from_slice(&output.stdout)?;
    println!("🎵 detected: {}", info.title.clone().white().bold());

    let type_options = vec!["audio only", "video only", "both (audio + video)"];
    let type_idx = Select::new()
        .with_prompt("what would you like to download?")
        .items(&type_options)
        .default(0)
        .interact()?;

    let mut downloads = Vec::new();

    // 1. Audio Configuration
    if type_idx == 0 || type_idx == 2 {
        println!("\n--- [AUDIO CONFIGURATION] ---");
        let audio_formats = vec!["mp3", "m4a", "wav", "flac", "opus"];
        let selected_indices = MultiSelect::new()
            .with_prompt("select audio formats (space to toggle)")
            .items(&audio_formats)
            .interact()?;
        
        if !selected_indices.is_empty() {
            let filename: String = Input::new()
                .with_prompt("audio base filename")
                .default(info.title.clone())
                .interact_text()?;

            for &idx in &selected_indices {
                downloads.push(DownloadConfig {
                    format: audio_formats[idx].to_string(),
                    filename: filename.clone(),
                    is_audio: true,
                    quality: None,
                });
            }
        }
    }

    // 2. Video Configuration
    if type_idx == 1 || type_idx == 2 {
        println!("\n--- [VIDEO CONFIGURATION] ---");
        let video_formats = vec!["mp4", "mkv", "webm"];
        let selected_indices = MultiSelect::new()
            .with_prompt("select video containers (space to toggle)")
            .items(&video_formats)
            .interact()?;

        if !selected_indices.is_empty() {
            // Extract resolutions
            let mut res_options: Vec<String> = info.formats.iter()
                .filter(|f| f.vcodec.as_deref().unwrap_or("none") != "none")
                .filter_map(|f| f.resolution.clone())
                .collect();
            res_options.sort();
            res_options.dedup();
            res_options.push("best".to_string());

            let res_idx = Select::new()
                .with_prompt("video quality (max resolution)")
                .items(&res_options)
                .default(res_options.len() - 1)
                .interact()?;

            let filename: String = Input::new()
                .with_prompt("video base filename")
                .default(info.title.clone())
                .interact_text()?;

            for &idx in &selected_indices {
                downloads.push(DownloadConfig {
                    format: video_formats[idx].to_string(),
                    filename: filename.clone(),
                    is_audio: false,
                    quality: Some(res_options[res_idx].clone()),
                });
            }
        }
    }

    if downloads.is_empty() {
        println!("⚠️ no formats selected. exiting.");
        return Ok(());
    }

    // 3. Shared Options
    println!("\n--- [SHARED OPTIONS] ---");
    let meta_options = vec!["embed metadata", "embed thumbnail", "embed subtitles"];
    let meta_selected = MultiSelect::new()
        .with_prompt("additional features (space to toggle)")
        .items(&meta_options)
        .interact()?;

    let mut common_args = Vec::new();
    for idx in meta_selected {
        match idx {
            0 => common_args.push("--embed-metadata".to_string()),
            1 => common_args.push("--embed-thumbnail".to_string()),
            2 => common_args.push("--embed-subs".to_string()),
            _ => {}
        }
    }

    // 4. Execution
    for config in downloads {
        println!("\n🚀 downloading [{}]: {}", config.format.clone().cyan(), config.filename.clone().white().bold());
        let mut cmd = Command::new("yt-dlp");
        cmd.args(&common_args);
        
        if config.is_audio {
            cmd.args(["-x", "--audio-format", &config.format, "--audio-quality", "0"]);
        } else {
            let res_filter = if let Some(q) = config.quality {
                if q == "best" { "bv+ba/b".to_string() } else { format!("bv[height<={pos}]+ba/b[height<={pos}]", pos = q.split('x').last().unwrap_or("1080")) }
            } else { "bv+ba/b".to_string() };
            cmd.args(["-f", &res_filter, "--merge-output-format", &config.format]);
        }

        cmd.args(["-o", &format!("{}.%(ext)s", config.filename)]);
        cmd.arg(&url);
        
        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            println!("❌ failed to download: {} ({})", config.filename, config.format);
        }
    }

    println!("\n✅ all operations complete.");
    Ok(())
}

fn render_ui(mpv: &Mpv, is_loop: bool, show_lyrics: bool) -> Result<()> {
    let mut stdout = io::stdout();
    let (width, _) = terminal::size().unwrap_or((80, 24));
    let width = width as usize;

    let time = mpv.get_property::<f64>("time-pos").unwrap_or(0.0);
    let duration = mpv.get_property::<f64>("duration").unwrap_or(0.0);
    let paused = mpv.get_property::<bool>("pause").unwrap_or(false);
    let mute = mpv.get_property::<bool>("mute").unwrap_or(false);
    let volume = mpv.get_property::<f64>("volume").unwrap_or(0.0);
    let speed = mpv.get_property::<f64>("speed").unwrap_or(1.0);
    let title = mpv.get_property::<String>("media-title").unwrap_or_else(|_| "...".to_string());
    let bitrate = mpv.get_property::<f64>("audio-bitrate").unwrap_or(0.0) / 1000.0;

    // Integrated Status String
    let status_base = if paused { "paused" } else { "playing" };
    let status_str = if mute {
        format!("{} | mute", status_base)
    } else {
        status_base.to_string()
    };

    let loop_tag = if is_loop { " | loop" } else { "" };
    let tech_tags = format!(" | {:.0}kbps{}", bitrate, loop_tag);

    queue!(stdout, cursor::MoveToColumn(0))?;

    // --- Line 1 ---
    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let status_styled = format!("[{}] ", status_str);
    let available_for_title = width.saturating_sub(status_styled.len() + tech_tags.len() + 5);
    
    let display_title = if title.len() > available_for_title {
        format!("{}...", &title.chars().take(available_for_title.saturating_sub(3)).collect::<String>())
    } else {
        title
    };

    print!("{}{}{}", status_styled.yellow().bold(), display_title.white().bold(), tech_tags.cyan());
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    // --- Line 2 ---
    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let progress = if duration > 0.0 { time / duration } else { 0.0 };
    let time_str = format!("{:02}:{:02} / {:02}:{:02}", (time/60.) as i32, (time%60.) as i32, (duration/60.) as i32, (duration%60.) as i32);
    let specs_str = format!(" | volume [↑/↓]: {:>3.0}% | speed [+/-]: {:.1}x", volume, speed);
    
    let bar_width = width.saturating_sub(time_str.len() + specs_str.len() + 4).max(10);
    let filled = (progress * bar_width as f64) as usize;
    
    print!("{} | ", time_str.dark_grey());
    print!("{}", "━".repeat(filled).white());
    print!("{}", "─".repeat(bar_width.saturating_sub(filled)).dark_grey());
    print!("{}", specs_str.white());
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    // --- Line 3 ---
    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    if show_lyrics {
        let lyrics = mpv.get_property::<String>("sub-text").unwrap_or_default().replace('\n', " ");
        if !lyrics.is_empty() {
            let lyrics_clean = lyrics.to_lowercase();
            let lyrics_display = if lyrics_clean.len() > width {
                format!("{}...", &lyrics_clean[..width.saturating_sub(4)])
            } else {
                lyrics_clean
            };
            print!("{}", lyrics_display.white().bold());
        } else {
            print!("{}", "---".dark_grey());
        }
    } else {
        let shortcuts = "[space] pause | [←/→] seek | [↑/↓] volume | [+/-] speed | [v] lyrics | [m] mute | [q] quit";
        print!("{}", if shortcuts.len() >= width { &shortcuts[..width.saturating_sub(1)] } else { shortcuts }.dark_grey());
    }
    
    queue!(stdout, cursor::MoveToColumn(0), cursor::MoveUp(2))?;
    stdout.flush()?;
    Ok(())
}

fn play_loop(mpv: &mut Mpv, mut is_loop: bool, mut show_lyrics: bool) -> Result<()> {
    loop {
        if let Some(event_result) = mpv.wait_event(0.0) {
            match event_result.map_err(|e| anyhow::anyhow!("mpv error: {:?}", e))? {
                MEvent::EndFile(_reason) => if !is_loop { 
                    let remaining: i64 = mpv.get_property("playlist-count").unwrap_or(0);
                    let pos: i64 = mpv.get_property("playlist-pos").unwrap_or(0);
                    if pos + 1 >= remaining || pos < 0 { break; }
                },
                MEvent::Shutdown => break,
                _ => {}
            }
        }

        if event::poll(Duration::from_millis(0))? {
            if let CEvent::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('p') | KeyCode::Char(' ') => {
                        let p: bool = mpv.get_property("pause").unwrap_or(false);
                        mpv.set_property("pause", !p).ok();
                    }
                    KeyCode::Char('m') => {
                        let m: bool = mpv.get_property("mute").unwrap_or(false);
                        mpv.set_property("mute", !m).ok();
                    }
                    KeyCode::Char('v') => {
                        show_lyrics = !show_lyrics;
                    }
                    KeyCode::Char('l') => {
                        is_loop = !is_loop;
                        mpv.set_property("loop-file", if is_loop { "inf" } else { "no" }).ok();
                    }
                    KeyCode::Char('0') => { mpv.set_property("speed", 1.0).ok(); }
                    KeyCode::Right => { mpv.command("seek", &["5", "relative"]).ok(); }
                    KeyCode::Left | KeyCode::Char('h') => { mpv.command("seek", &["-5", "relative"]).ok(); }
                    KeyCode::Char(']') => { mpv.command("seek", &["60", "relative"]).ok(); }
                    KeyCode::Char('[') => { mpv.command("seek", &["-60", "relative"]).ok(); }
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

        render_ui(mpv, is_loop, show_lyrics)?;
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
