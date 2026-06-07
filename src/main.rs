use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use libmpv2::Mpv;
use std::io;

mod audio;
mod discovery;
mod download;
mod help_topics;
mod player;
mod tactical_select;
mod tui;

use discovery::handle_search;
use download::handle_download;
use player::{PlayLoopResult, play_loop};

#[derive(Parser, Debug)]
#[command(
    name = "firemusic",
    author = "FireflyLabs",
    version = "0.2.6",
    about = "minimalist high-performance CLI music player",
    long_about = "FireMusic is a tactical CLI player for power users.\n\
                  Zero-leak interface, advanced playlist logic, built-in EQ,\n\
                  and multi-format download via yt-dlp.",
    help_template = "\x1b[1;33m{name}\x1b[0m v{version} \u{2014} {about}\n\n\
                     \x1b[1mUSAGE:\x1b[0m\n    {usage}\n\n\
                     \x1b[1mOPTIONS:\x1b[0m\n{options}\n\
                     \n\
                     \x1b[1mCOMMANDS:\x1b[0m\n\
                     \x20   help discovery        Search across providers\n\
                     \x20   help download         Download audio or video\n\
                     \x20   help interface        TUI and interface options\n\
                     \x20   help controls         Playback keyboard controls\n",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Args {
    #[arg(
        short = 'h',
        visible_short_alias = 'H',
        long,
        action = clap::ArgAction::Help,
        help = "Show help"
    )]
    help: Option<bool>,

    #[arg(
        required_unless_present = "download",
        required_unless_present = "search",
        required_unless_present = "tui",
        num_args = 0..,
        value_name = "INPUT"
    )]
    inputs: Vec<String>,

    #[arg(
        short,
        long = "loop",
        alias = "loop-mode",
        help = "Enable infinite playback"
    )]
    loop_mode: bool,

    #[arg(
        short = 'f',
        long,
        default_value_t = 1.0,
        value_name = "FACTOR",
        help = "Set playback speed factor"
    )]
    speed: f64,

    #[arg(
        short,
        long,
        default_value_t = 100.0,
        value_name = "LEVEL",
        help = "Set volume level"
    )]
    volume: f64,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        value_name = "MODE",
        require_equals = true,
        default_missing_value = "interactive",
        help = "Download media"
    )]
    download: Option<String>,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        value_name = "QUERY",
        require_equals = false,
        default_missing_value = "",
        help = "Search and play music"
    )]
    search: Option<String>,

    #[arg(
        short = 't',
        long = "tui",
        default_value_t = false,
        help = "Launch terminal user interface"
    )]
    tui: bool,

    #[arg(
        short = 'c',
        long = "crossfade",
        default_value_t = 0.0,
        value_name = "SECONDS",
        help = "Set crossfade duration"
    )]
    crossfade: f64,

    #[arg(
        short = 'm',
        long = "music-dir",
        default_value = "",
        value_name = "DIR",
        hide_default_value = true,
        help = "Set local music library path"
    )]
    music_dir: String,
}

#[derive(Subcommand, Debug)]
enum HelpTopicCmd {
    /// Search across providers
    Discovery,
    /// Download audio or video
    Download,
    /// TUI and interface options
    Interface,
    /// Playback keyboard controls
    Controls,
}

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().collect();
    if raw.len() >= 2 && raw[1] == "help" {
        if raw.len() == 2 {
            Args::command().name("msc").print_help().ok();
            println!();
            return Ok(());
        }
        #[derive(Parser)]
        #[command(name = "msc help")]
        struct HelpCli {
            #[command(subcommand)]
            topic: HelpTopicCmd,
        }
        let help_args: Vec<String> = std::iter::once("msc help".to_string())
            .chain(raw[2..].iter().cloned())
            .collect();
        let cli = HelpCli::try_parse_from(&help_args).unwrap_or_else(|e| e.exit());
        match cli.topic {
            HelpTopicCmd::Discovery => help_topics::discovery(),
            HelpTopicCmd::Download => help_topics::download(),
            HelpTopicCmd::Interface => help_topics::interface(),
            HelpTopicCmd::Controls => help_topics::controls(),
        }
        return Ok(());
    }

    let args = Args::parse();

    if args.tui {
        return tui::run_tui(
            args.inputs,
            args.crossfade,
            args.loop_mode,
            args.volume,
            args.speed,
            &args.music_dir,
        );
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

    execute!(
        stdout,
        cursor::Show,
        cursor::MoveToColumn(0),
        cursor::MoveDown(3)
    )?;
    println!();
    disable_raw_mode().ok();

    if let Ok(PlayLoopResult::Quit) = res {
        println!(
            "{}",
            "\u{2500}\u{2500} fire music session closed \u{2500}\u{2500}".dark_grey()
        );
    }
    Ok(())
}
