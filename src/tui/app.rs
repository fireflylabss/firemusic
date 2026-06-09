use crate::core::audio::crossfade::CrossfadeConfig;
use crate::core::config::{default_music_dir, playlists_dir};
use std::path::PathBuf;

pub const EQ_PRESETS: &[&str] = &[
    "",                                             // off
    "bass=g=10",                                    // bass+
    "treble=g=10",                                  // treble+
    "bass=g=10,treble=g=10",                        // rock
    "equalizer=f=1000:width_type=h:width=200:g=10", // vocal
    "equalizer=f=300:width_type=h:width=200:g=-10,equalizer=f=3000:width_type=h:width=200:g=-10", // lofi
    "bass=g=8,treble=g=5",                                            // pop
    "bass=g=5,treble=g=3,equalizer=f=500:width_type=h:width=200:g=3", // jazz
    "equalizer=f=2000:width_type=h:width=200:g=5,equalizer=f=4000:width_type=h:width=200:g=5,equalizer=f=8000:width_type=h:width=200:g=5", // classical
    "bass=g=12,treble=g=8",                                   // electronic
    "bass=g=12,equalizer=f=1000:width_type=h:width=200:g=-3", // hiphop
];

pub const EQ_PRESET_NAMES: &[&str] = &[
    "off",
    "bass+",
    "treble+",
    "rock",
    "vocal",
    "lofi",
    "pop",
    "jazz",
    "classical",
    "electronic",
    "hiphop",
];

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub path: String,
    #[allow(dead_code)]
    pub duration: f64,
    #[allow(dead_code)]
    pub artist: Option<String>,
    #[allow(dead_code)]
    pub album: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Queue,
    Library,
    Playlists,
    Stats,
}

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Queue => "Queue",
            Tab::Library => "Library",
            Tab::Playlists => "Playlists",
            Tab::Stats => "Stats",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    NowPlaying,
}

impl Focus {
    pub fn label(self) -> &'static str {
        match self {
            Focus::List => "list",
            Focus::NowPlaying => "now playing",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LibraryEntry {
    Folder(String),
    Track(Track),
}

#[derive(Debug, Clone)]
pub struct LibraryState {
    pub root_dir: PathBuf,
    pub current_dir: PathBuf,
    pub entries: Vec<LibraryEntry>,
    pub selected_idx: usize,
    pub filter: String,
}

const AUDIO_EXTS: &[&str] = &["mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "wma"];

impl LibraryState {
    pub fn new() -> Self {
        Self::with_root(default_music_dir())
    }
    pub fn with_root(root: PathBuf) -> Self {
        let mut state = Self {
            root_dir: root.clone(),
            current_dir: root,
            entries: Vec::new(),
            selected_idx: 0,
            filter: String::new(),
        };
        state.scan_current_dir();
        state
    }
    pub fn scan_current_dir(&mut self) {
        self.entries.clear();
        self.selected_idx = 0;
        self.filter.clear();
        if !self.current_dir.exists() || !self.current_dir.is_dir() {
            return;
        }
        let mut folders: Vec<String> = Vec::new();
        let mut tracks: Vec<Track> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    folders.push(entry.file_name().to_string_lossy().to_string());
                } else if let Some(ext) = entry.path().extension() {
                    if AUDIO_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                        let title = entry
                            .path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                        tracks.push(Track {
                            title,
                            path: entry.path().to_string_lossy().to_string(),
                            duration: 0.0,
                            artist: None,
                            album: None,
                        });
                    }
                }
            }
        }
        folders.sort();
        tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        for f in folders {
            self.entries.push(LibraryEntry::Folder(f));
        }
        for t in tracks {
            self.entries.push(LibraryEntry::Track(t));
        }
    }

    pub fn scan_current_dir_with_callback<F: FnMut(bool, &str)>(&mut self, mut callback: F) {
        self.entries.clear();
        self.selected_idx = 0;
        self.filter.clear();
        if !self.current_dir.exists() || !self.current_dir.is_dir() {
            return;
        }

        callback(true, "scanning directory");

        let mut folders: Vec<String> = Vec::new();
        let mut tracks: Vec<Track> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    folders.push(entry.file_name().to_string_lossy().to_string());
                } else if let Some(ext) = entry.path().extension() {
                    if AUDIO_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                        let title = entry
                            .path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                        tracks.push(Track {
                            title,
                            path: entry.path().to_string_lossy().to_string(),
                            duration: 0.0,
                            artist: None,
                            album: None,
                        });
                    }
                }
            }
        }
        folders.sort();
        tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        for f in folders {
            self.entries.push(LibraryEntry::Folder(f));
        }
        for t in tracks {
            self.entries.push(LibraryEntry::Track(t));
        }

        callback(false, "scan complete");
    }
    pub fn enter_folder(&mut self) {
        if let Some(LibraryEntry::Folder(name)) = self.entries.get(self.selected_idx) {
            self.current_dir = self.current_dir.join(name);
            self.scan_current_dir();
        }
    }
    pub fn go_to_parent(&mut self) {
        if self.current_dir != self.root_dir {
            if let Some(parent) = self.current_dir.parent() {
                let parent = parent.to_path_buf();
                self.current_dir =
                    if parent.starts_with(&self.root_dir) || self.root_dir.starts_with(&parent) {
                        parent
                    } else {
                        self.root_dir.clone()
                    };
                self.scan_current_dir();
            }
        }
    }
    pub fn change_root(&mut self, new_root: PathBuf) {
        self.root_dir = new_root.clone();
        self.current_dir = new_root;
        self.scan_current_dir();
    }
    pub fn displayed_path(&self) -> String {
        let r = self.root_dir.to_string_lossy();
        let c = self.current_dir.to_string_lossy();
        if c == r {
            abbreviate_home(&r)
        } else if c.starts_with(r.as_ref()) {
            c[r.len()..].trim_start_matches('/').to_string()
        } else {
            abbreviate_home(&c)
        }
    }

    pub fn visible_entries(&self) -> Vec<(usize, &LibraryEntry)> {
        if self.filter.trim().is_empty() {
            return self.entries.iter().enumerate().collect();
        }
        let filter = self.filter.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| match entry {
                LibraryEntry::Folder(name) => name.to_lowercase().contains(&filter),
                LibraryEntry::Track(track) => track.title.to_lowercase().contains(&filter),
            })
            .collect()
    }
}

fn abbreviate_home(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home = home.to_string_lossy();
        if path == home {
            return "~".to_string();
        }
        if let Some(rest) = path.strip_prefix(&format!("{}/", home)) {
            return format!("~/{}", rest);
        }
    }
    path.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    ChangeLibraryRoot,
    NewPlaylist,
    SavePlaylist,
    FilterLibrary,
}

#[derive(Debug, Clone)]
pub struct InputMode {
    pub action: InputAction,
    pub prompt: String,
    pub value: String,
}

pub struct PlaylistManager {
    pub playlists: Vec<String>,
    pub current_tracks: Vec<Track>,
    pub selected_idx: usize,
}

impl PlaylistManager {
    pub fn new() -> Self {
        let mut pm = Self {
            playlists: Vec::new(),
            current_tracks: Vec::new(),
            selected_idx: 0,
        };
        pm.refresh();
        pm
    }
    pub fn refresh(&mut self) {
        self.playlists.clear();
        let dir = playlists_dir();
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if let Some(n) = entry.file_name().to_str() {
                        if n.ends_with(".m3u") {
                            self.playlists.push(n.trim_end_matches(".m3u").to_string());
                        }
                    }
                }
            }
        }
        self.playlists.sort();
    }
    pub fn load_playlist(&mut self, name: &str) {
        self.current_tracks.clear();
        let path = playlists_dir().join(format!("{}.m3u", name));
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content
                .lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            {
                let title = std::path::Path::new(line)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| line.to_string());
                self.current_tracks.push(Track {
                    title,
                    path: line.to_string(),
                    duration: 0.0,
                    artist: None,
                    album: None,
                });
            }
        }
    }
    #[allow(dead_code)]
    pub fn save_playlist(&self, name: &str, tracks: &[Track]) -> std::io::Result<()> {
        let dir = playlists_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.m3u", name));
        std::fs::write(
            &path,
            tracks
                .iter()
                .map(|t| format!("{}\n", t.path))
                .collect::<String>(),
        )
    }
    pub fn delete_playlist(name: &str) -> std::io::Result<()> {
        std::fs::remove_file(playlists_dir().join(format!("{}.m3u", name)))
    }
    pub fn total_items(&self) -> usize {
        if self.current_tracks.is_empty() {
            self.playlists.len()
        } else {
            self.current_tracks.len()
        }
    }
}

pub struct AppState {
    pub queue: Vec<Track>,
    pub current_track_idx: usize,
    pub queue_cursor: usize,
    pub active_tab: Tab,
    pub focus: Focus,
    pub library: LibraryState,
    pub playlists: PlaylistManager,
    pub crossfade: CrossfadeConfig,
    pub playback: PlaybackState,
    pub status_msg: Option<String>,
    pub status_msg_time: Option<std::time::Instant>,
    pub eq_preset: usize,
    pub loading: bool,
    pub loading_msg: String,
    pub show_help_popup: bool,
    pub input_mode: Option<InputMode>,
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub time: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub speed: f64,
    pub pitch: f64,
    pub title: String,
    pub bitrate_kbps: f64,
    pub is_loop: bool,
    pub cover_art: Option<Vec<u8>>,
    pub cover_id: u32,
    pub cover_w: u32,
    pub cover_h: u32,
    pub last_cover_path: String,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            duration: 0.0,
            paused: false,
            muted: false,
            volume: 100.0,
            speed: 1.0,
            pitch: 1.0,
            title: "...".to_string(),
            bitrate_kbps: 0.0,
            is_loop: false,
            cover_art: None,
            cover_id: 1,
            cover_w: 0,
            cover_h: 0,
            last_cover_path: String::new(),
        }
    }
}

impl AppState {
    pub fn new(crossfade_duration: f64, is_loop: bool, music_dir: PathBuf) -> Self {
        let crossfade = if crossfade_duration > 0.0 {
            CrossfadeConfig::new(crossfade_duration)
        } else {
            CrossfadeConfig::disabled()
        };
        let mut pb = PlaybackState::new();
        pb.is_loop = is_loop;
        let library = LibraryState::with_root(music_dir);
        Self {
            queue: Vec::new(),
            current_track_idx: 0,
            queue_cursor: 0,
            active_tab: Tab::Queue,
            focus: Focus::List,
            library,
            playlists: PlaylistManager::new(),
            crossfade,
            playback: pb,
            eq_preset: 0,
            status_msg: Some("F1-F4 tabs  Tab focus  ? help  q quit".to_string()),
            status_msg_time: Some(std::time::Instant::now()),
            loading: false,
            loading_msg: String::new(),
            show_help_popup: false,
            input_mode: None,
        }
    }

    pub fn set_message(&mut self, msg: String) {
        self.status_msg = Some(msg);
        self.status_msg_time = Some(std::time::Instant::now());
    }

    pub fn start_input(
        &mut self,
        action: InputAction,
        prompt: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.input_mode = Some(InputMode {
            action,
            prompt: prompt.into(),
            value: value.into(),
        });
        self.status_msg = None;
        self.status_msg_time = None;
    }

    pub fn clear_old_message(&mut self) {
        if self.input_mode.is_some() {
            return;
        }
        if let Some(time) = self.status_msg_time {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.status_msg = None;
                self.status_msg_time = None;
            }
        }
    }
}
