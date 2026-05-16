use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    style::Stylize,
};
use libmpv2::Mpv;
use std::io;

mod tactical_select;
mod player;
mod discovery;
mod download;
mod audio;
mod tui;

use player::{play_loop, PlayLoopResult};
use discovery::handle_search;
use download::handle_download;

#[derive(Parser, Debug)]
#[command(
    name = "firemusic",
    author = "FireflyLabs",
    version = "0.3.0",
    about = "fire music - minimalist high-performance cli music player",
    long_about = "fire music is a tactical cli player designed for pro users. \
                  it features a 'zero-leak' interface, advanced playlist logic, \
                  and an integrated multi-format download system via yt-dlp.",
    help_template = "\x1b[1m{name} v{version}\x1b[0m - {about}\n\n\
                     \x1b[1m{usage-heading}\x1b[0m {usage}\n\n\
                     \x1b[1mOPTIONS:\x1b[0m\n\
                     {options}\n\n\
                     \x1b[1mCONTROLS:\x1b[0m\n\
                       space                pause / resume\n\
                       \u{2190} / \u{2192}, h / l         seek 5s\n\
                       {{ / }}                seek 1m\n\
                       \u{2191} / \u{2193}, k / j         volume +/-\n\
                       1 - 9                jump to % (10-90%)\n\
                       0                    reset speed/pitch/eq\n\
                       + / -                speed +/-\n\
                       , / .                pitch down / up\n\
                       e                    cycle equalizer\n\
                       E                    eq mode (per-band adjustment)\n\
                       l                    toggle loop mode\n\
                       m                    toggle mute\n\
                       s                    back to search menu (at end)\n\
                       q, esc               quit session\n\n\
                     \x1b[1mMODES:\x1b[0m\n\
                       msc --tui            full terminal interface (F1-F3 tabs, Tab focus)\n\
                       msc --tui -M ~/music custom library scan dir\n\n\
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
                       msc --tui             open full terminal interface\n\
                       msc -s \"not like us\"  search and play across providers\n",
    disable_help_subcommand = true,
)]
struct Args {
    #[arg(
        required_unless_present = "download",
        required_unless_present = "search",
        required_unless_present = "tui",
        num_args = 0..,
        value_name = "INPUT"
    )]
    inputs: Vec<String>,

    #[arg(short, long = "loop", alias = "loop-mode", help = "enable infinite loop mode")]
    loop_mode: bool,

    #[arg(short = 'f', long, default_value_t = 1.0, value_name = "F", help = "set initial speed factor")]
    speed: f64,

    #[arg(short, long, default_value_t = 100.0, value_name = "L", help = "set initial volume level")]
    volume: f64,

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

    #[arg(
        short = 't',
        long = "tui",
        default_value_t = false,
        help = "launch full terminal user interface"
    )]
    tui: bool,

    #[arg(
        short = 'c',
        long = "crossfade",
        default_value_t = 0.0,
        value_name = "SECONDS",
        help = "crossfade duration between tracks in seconds (0 = disabled)"
    )]
    crossfade: f64,

    #[arg(
        short = 'M',
        long = "music-dir",
        default_value = "",
        value_name = "DIR",
        help = "directory to scan for local music library (default: ~/Music)"
    )]
    music_dir: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.tui {
        return tui::run_tui(args.inputs, args.crossfade, args.loop_mode, args.volume, args.speed, &args.music_dir);
    }

    if let Some(mode) = args.download {
        return handle_download(&mode, args.inputs);
    }

    if let Some(query) = args.search {
        return handle_search(query, args.loop_mode, args.volume, args.speed);
    }

    let mut mpv = Mpv::new().map_err(|e| anyhow::anyhow!("mpv init error: {:?}", e))?;

    mpv.set_property("video", "no").ok();
    mpv.set_property("volume", args.volume).ok();
    mpv.set_property("speed", args.speed).ok();
    mpv.set_property("ytdl", "yes").ok();
    mpv.set_property("ytdl-format", "bestaudio/best").ok();

    let is_loop = args.loop_mode;
    if is_loop {
        mpv.set_property("loop-file", "inf").ok();
    }

    if args.crossfade > 0.0 {
        mpv.set_property("audio-fade", args.crossfade).ok();
    }

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
        println!("{}", "\u{2500}\u{2500} fire music session closed \u{2500}\u{2500}".dark_grey());
    }
    Ok(())
}
