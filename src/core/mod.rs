pub mod audio;
pub mod discovery;
pub mod download;
pub mod mpv;
pub mod paths;
pub mod player;
pub mod tactical_select;

pub use discovery::{handle_search, SearchResult, SearchProvider, YtdlInfo, PROVIDERS};
pub use download::handle_download;
pub use mpv::{create_player, load_inputs, MpvConfig};
pub use paths::{config_dir, resolve_music_dir, validate_playback_input, validate_playback_inputs, validate_url};
pub use player::{eq_mode_overlay, play_loop, render_ui, PlayLoopResult};
pub use tactical_select::tactical_select;