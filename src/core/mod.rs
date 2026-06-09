pub mod audio;
pub mod config;
pub mod discovery;
pub mod download;
pub mod error;
pub mod mpv;
pub mod paths;
pub mod player;
pub mod store;
pub mod tactical_select;

pub use config::{config_dir, default_music_dir, playlists_dir, presets_dir, resolve_music_dir};
pub use discovery::{handle_search, SearchResult, SearchProvider, YtdlInfo, PROVIDERS};
pub use download::handle_download;
pub use error::Error;
pub use mpv::{create_player, load_inputs, MpvConfig};
pub use paths::{validate_playback_input, validate_playback_inputs, validate_url};
pub use player::{eq_mode_overlay, play_loop, render_ui, PlayLoopResult};
pub use tactical_select::tactical_select;