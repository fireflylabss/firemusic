use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "firemusic",
    author = "FireflyLabs",
    version = "0.2.9",
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
pub struct Args {
    #[arg(
        short = 'h',
        visible_short_alias = 'H',
        long,
        action = clap::ArgAction::Help,
        help = "Show help"
    )]
    pub help: Option<bool>,

    #[arg(
        required_unless_present = "download",
        required_unless_present = "search",
        required_unless_present = "tui",
        num_args = 0..,
        value_name = "INPUT"
    )]
    pub inputs: Vec<String>,

    #[arg(
        short,
        long = "loop",
        alias = "loop-mode",
        help = "Enable infinite playback"
    )]
    pub loop_mode: bool,

    #[arg(
        short = 'f',
        long,
        default_value_t = 1.0,
        value_name = "FACTOR",
        help = "Set playback speed factor"
    )]
    pub speed: f64,

    #[arg(
        short,
        long,
        default_value_t = 100.0,
        value_name = "LEVEL",
        help = "Set volume level"
    )]
    pub volume: f64,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        value_name = "MODE",
        require_equals = true,
        default_missing_value = "interactive",
        help = "Download media"
    )]
    pub download: Option<String>,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        value_name = "QUERY",
        require_equals = false,
        default_missing_value = "",
        help = "Search and play music"
    )]
    pub search: Option<String>,

    #[arg(
        short = 't',
        long = "tui",
        default_value_t = false,
        help = "Launch terminal user interface"
    )]
    pub tui: bool,

    #[arg(
        short = 'c',
        long = "crossfade",
        default_value_t = 0.0,
        value_name = "SECONDS",
        help = "Set crossfade duration"
    )]
    pub crossfade: f64,

    #[arg(
        short = 'm',
        long = "music-dir",
        default_value = "",
        value_name = "DIR",
        hide_default_value = true,
        help = "Set local music library path"
    )]
    pub music_dir: String,
}

#[derive(Subcommand, Debug)]
pub enum HelpTopicCmd {
    /// Search across providers
    Discovery,
    /// Download audio or video
    Download,
    /// TUI and interface options
    Interface,
    /// Playback keyboard controls
    Controls,
}