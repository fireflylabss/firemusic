use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDto {
    pub title: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LibraryEntryDto {
    Folder { name: String },
    Track { title: String, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybackDto {
    pub title: String,
    pub time: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub speed: f64,
    pub is_loop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuiSnapshot {
    pub tab: String,
    pub queue: Vec<TrackDto>,
    pub queue_cursor: usize,
    pub current_track_idx: usize,
    pub library_path: String,
    pub library_entries: Vec<LibraryEntryDto>,
    pub library_selected: usize,
    pub library_is_root: bool,
    pub library_filter: String,
    pub playlists: Vec<String>,
    pub playlist_tracks: Vec<TrackDto>,
    pub playlist_viewing: bool,
    pub playlist_selected: usize,
    pub playback: PlaybackDto,
    pub status_msg: Option<String>,
    pub cover_base64: Option<String>,
}