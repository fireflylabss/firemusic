pub mod cli;
pub mod core;
pub mod gui;
pub mod tui;

pub use core::{
    create_player, handle_download, handle_search, play_loop, resolve_music_dir,
    validate_playback_inputs, MpvConfig, PlayLoopResult,
};