use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::core::{
    create_player, load_inputs, play_loop, validate_playback_inputs, MpvConfig, PlayLoopResult,
};

pub fn run_direct_play(
    inputs: &[String],
    loop_mode: bool,
    volume: f64,
    speed: f64,
    crossfade: f64,
) -> Result<()> {
    let validated = validate_playback_inputs(inputs)?;
    let config = MpvConfig::for_cli(volume, speed, loop_mode, crossfade);
    let mut mpv = create_player(&config)?;
    load_inputs(&mpv, &validated)?;

    enable_raw_mode().context("terminal error")?;
    let mut stdout = io::stdout();
    println!("\n\n\n");
    execute!(stdout, cursor::Hide, cursor::MoveUp(3))?;

    let res = play_loop(&mut mpv, loop_mode);

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