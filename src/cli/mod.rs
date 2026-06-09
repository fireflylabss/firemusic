mod args;
mod help_topics;
mod play;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use args::{Args, HelpTopicCmd};

use crate::core::{
    handle_download, handle_search, resolve_music_dir, validate_playback_inputs,
};
use crate::tui;

pub fn run() -> Result<()> {
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

    if args.gui {
        let music_dir = resolve_music_dir(&args.music_dir)?;
        let inputs = if args.inputs.is_empty() {
            Vec::new()
        } else {
            validate_playback_inputs(&args.inputs)?
        };
        return crate::gui::launch_gui(
            music_dir,
            inputs,
            args.crossfade,
            args.loop_mode,
            args.volume,
            args.speed,
        );
    }

    if args.tui {
        let music_dir = resolve_music_dir(&args.music_dir)?;
        let inputs = if args.inputs.is_empty() {
            Vec::new()
        } else {
            validate_playback_inputs(&args.inputs)?
        };
        return tui::run_tui(
            inputs,
            args.crossfade,
            args.loop_mode,
            args.volume,
            args.speed,
            music_dir,
        );
    }

    if let Some(mode) = args.download {
        return handle_download(&mode, args.inputs);
    }

    if let Some(query) = args.search {
        return handle_search(query, args.loop_mode, args.volume, args.speed);
    }

    play::run_direct_play(
        &args.inputs,
        args.loop_mode,
        args.volume,
        args.speed,
        args.crossfade,
    )
}