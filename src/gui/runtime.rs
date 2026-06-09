use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use libmpv2::{Mpv, events::Event as MEvent};

use crate::core::{create_player, load_inputs, validate_playback_inputs, MpvConfig};
use crate::tui::app::{AppState, LibraryEntry, Tab, Track};

use super::cover::{extract_cover_art, png_dimensions};
use super::snapshot::{
    GuiSnapshot, LibraryEntryDto, PlaybackDto, TrackDto,
};

pub enum GuiCommand {
    SetTab(Tab),
    LibrarySelect(usize),
    LibraryEnter,
    LibraryBack,
    LibraryRescan,
    LibraryFilter(String),
    LibraryAddSelected,
    QueueSelect(usize),
    QueuePlay(usize),
    QueueRemove(usize),
    TogglePause,
    Seek(f64),
    SetVolume(f64),
    ToggleMute,
    ToggleLoop,
    PlaylistSelect(usize),
    PlaylistLoad,
    PlaylistBack,
    PlaylistRefresh,
    Shutdown,
}

pub struct GuiRuntime {
    cmd_tx: mpsc::Sender<GuiCommand>,
    snapshot: Arc<Mutex<GuiSnapshot>>,
    join: Mutex<Option<thread::JoinHandle<()>>>,
}

impl GuiRuntime {
    pub fn start(
        music_dir: PathBuf,
        inputs: Vec<String>,
        crossfade: f64,
        is_loop: bool,
        volume: f64,
        speed: f64,
    ) -> anyhow::Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let snapshot = Arc::new(Mutex::new(GuiSnapshot::default()));
        let snap_worker = Arc::clone(&snapshot);

        let join = thread::spawn(move || {
            if let Err(err) = worker_loop(
                music_dir,
                inputs,
                crossfade,
                is_loop,
                volume,
                speed,
                cmd_rx,
                snap_worker,
            ) {
                eprintln!("gui worker error: {err}");
            }
        });

        Ok(Self {
            cmd_tx,
            snapshot,
            join: Mutex::new(Some(join)),
        })
    }

    pub fn send(&self, cmd: GuiCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    pub fn snapshot(&self) -> GuiSnapshot {
        self.snapshot.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(GuiCommand::Shutdown);
        if let Ok(mut guard) = self.join.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }
    }
}

fn worker_loop(
    music_dir: PathBuf,
    inputs: Vec<String>,
    crossfade: f64,
    is_loop: bool,
    volume: f64,
    speed: f64,
    cmd_rx: mpsc::Receiver<GuiCommand>,
    snapshot: Arc<Mutex<GuiSnapshot>>,
) -> anyhow::Result<()> {
    let config = MpvConfig::for_cli(volume, speed, is_loop, crossfade);
    let mut mpv = create_player(&config)?;
    let mut state = AppState::new(crossfade, is_loop, music_dir);
    state.playback.volume = volume;
    state.playback.speed = speed;

    if !inputs.is_empty() {
        if let Ok(validated) = validate_playback_inputs(&inputs) {
            let _ = load_inputs(&mpv, &validated);
            for input in &validated {
                let title = track_title_from_path(input);
                state.queue.push(Track {
                    title,
                    path: input.clone(),
                    duration: 0.0,
                    artist: None,
                    album: None,
                });
            }
        }
    }

    let mut cover_bytes: Option<Vec<u8>> = None;
    let mut cover_path = String::new();
    let mut last_track: Option<String> = None;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            if matches!(cmd, GuiCommand::Shutdown) {
                return Ok(());
            }
            handle_command(&mpv, &mut state, cmd);
        }

        if let Some(event_result) = mpv.wait_event(0.0) {
            if let Ok(MEvent::EndFile(_)) = event_result {
                if !state.playback.is_loop
                    && state.current_track_idx + 1 < state.queue.len()
                {
                    state.current_track_idx += 1;
                    state.queue_cursor = state.current_track_idx;
                    let next = state.queue[state.current_track_idx].clone();
                    let _ = mpv.command("loadfile", &[&next.path, "replace"]);
                    state.set_message(format!("now playing: {}", next.title));
                }
            }
        }

        update_playback(&mpv, &mut state);

        let current = state
            .queue
            .get(state.current_track_idx)
            .map(|t| t.path.clone());
        if current != last_track {
            last_track = current.clone();
            cover_bytes = None;
            cover_path.clear();
            if let Some(path) = current {
                if !path.starts_with("http") {
                    if let Some(data) = extract_cover_art(&path) {
                        let (w, h) = png_dimensions(&data);
                        if w > 0 && h > 0 {
                            cover_bytes = Some(data);
                            cover_path = path;
                        }
                    }
                }
            }
        }

        let snap = build_snapshot(&state, &cover_bytes, &cover_path);
        if let Ok(mut guard) = snapshot.lock() {
            *guard = snap;
        }

        thread::sleep(Duration::from_millis(120));
    }
}

fn handle_command(mpv: &Mpv, state: &mut AppState, cmd: GuiCommand) {
    match cmd {
        GuiCommand::SetTab(tab) => state.active_tab = tab,
        GuiCommand::LibrarySelect(idx) => state.library.selected_idx = idx,
        GuiCommand::LibraryEnter => {
            let is_root = state.library.current_dir == state.library.root_dir;
            if !is_root && state.library.selected_idx == 0 {
                state.library.go_to_parent();
            } else {
                let vidx = if is_root {
                    state.library.selected_idx
                } else {
                    state.library.selected_idx.saturating_sub(1)
                };
                let visible = state.library.visible_entries();
                if let Some((eidx, entry)) = visible.get(vidx) {
                    let eidx = *eidx;
                    let entry = (*entry).clone();
                    match entry {
                        LibraryEntry::Folder(_) => {
                            state.library.selected_idx = eidx;
                            state.library.enter_folder();
                        }
                        LibraryEntry::Track(track) => add_track(mpv, state, &track),
                    }
                }
            }
        }
        GuiCommand::LibraryBack => state.library.go_to_parent(),
        GuiCommand::LibraryRescan => {
            state.library.scan_current_dir();
            state.set_message("library scanned".to_string());
        }
        GuiCommand::LibraryFilter(filter) => {
            state.library.filter = filter;
            state.library.selected_idx = 0;
        }
        GuiCommand::LibraryAddSelected => {
            let is_root = state.library.current_dir == state.library.root_dir;
            let vidx = if is_root {
                state.library.selected_idx
            } else {
                state.library.selected_idx.saturating_sub(1)
            };
            let track: Option<Track> = {
                let visible = state.library.visible_entries();
                visible.get(vidx).and_then(|(_, entry)| match entry {
                    LibraryEntry::Track(track) => Some(track.clone()),
                    _ => None,
                })
            };
            if let Some(track) = track {
                add_track(mpv, state, &track);
            }
        }
        GuiCommand::QueueSelect(idx) => state.queue_cursor = idx,
        GuiCommand::QueuePlay(idx) => {
            if idx < state.queue.len() {
                state.current_track_idx = idx;
                state.queue_cursor = idx;
                let track = &state.queue[idx];
                let _ = mpv.command("loadfile", &[&track.path, "replace"]);
                state.set_message(format!("playing: {}", track.title));
            }
        }
        GuiCommand::QueueRemove(idx) => {
            if idx < state.queue.len() {
                let removed = state.queue.remove(idx);
                if state.queue_cursor >= state.queue.len() && !state.queue.is_empty() {
                    state.queue_cursor = state.queue.len() - 1;
                }
                if state.current_track_idx >= state.queue.len() && !state.queue.is_empty() {
                    state.current_track_idx = state.queue.len() - 1;
                }
                state.set_message(format!("removed: {}", removed.title));
            }
        }
        GuiCommand::TogglePause => {
            let paused: bool = mpv.get_property("pause").unwrap_or(false);
            let _ = mpv.set_property("pause", !paused);
        }
        GuiCommand::Seek(delta) => {
            let pos: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
            let _ = mpv.set_property("time-pos", (pos + delta).max(0.0));
        }
        GuiCommand::SetVolume(vol) => {
            let v = vol.clamp(0.0, 100.0);
            let _ = mpv.set_property("volume", v);
            state.playback.volume = v;
        }
        GuiCommand::ToggleMute => {
            let muted: bool = mpv.get_property("mute").unwrap_or(false);
            let _ = mpv.set_property("mute", !muted);
        }
        GuiCommand::ToggleLoop => {
            state.playback.is_loop = !state.playback.is_loop;
            let _ = mpv.set_property(
                "loop-file",
                if state.playback.is_loop { "inf" } else { "no" },
            );
        }
        GuiCommand::PlaylistSelect(idx) => state.playlists.selected_idx = idx,
        GuiCommand::PlaylistLoad => {
            if state.playlists.current_tracks.is_empty() {
                if let Some(name) = state
                    .playlists
                    .playlists
                    .get(state.playlists.selected_idx)
                    .cloned()
                {
                    state.playlists.load_playlist(&name);
                    state.playlists.selected_idx = 0;
                }
            } else if let Some(track) = state
                .playlists
                .current_tracks
                .get(state.playlists.selected_idx)
                .cloned()
            {
                add_track(mpv, state, &track);
            }
        }
        GuiCommand::PlaylistBack => {
            state.playlists.current_tracks.clear();
            state.playlists.selected_idx = 0;
        }
        GuiCommand::PlaylistRefresh => state.playlists.refresh(),
        GuiCommand::Shutdown => {}
    }
}

fn add_track(mpv: &Mpv, state: &mut AppState, track: &Track) {
    state.queue.push(track.clone());
    if state.queue.len() == 1 {
        state.current_track_idx = 0;
        state.queue_cursor = 0;
        let _ = mpv.command("loadfile", &[&track.path, "replace"]);
    } else {
        let _ = mpv.command("loadfile", &[&track.path, "append-play"]);
    }
    state.set_message(format!("added: {}", track.title));
}

fn update_playback(mpv: &Mpv, state: &mut AppState) {
    state.playback.time = mpv.get_property("time-pos").unwrap_or(0.0);
    state.playback.duration = mpv.get_property("duration").unwrap_or(0.0);
    state.playback.paused = mpv.get_property("pause").unwrap_or(false);
    state.playback.muted = mpv.get_property("mute").unwrap_or(false);
    state.playback.volume = mpv.get_property("volume").unwrap_or(100.0);
    state.playback.speed = mpv.get_property("speed").unwrap_or(1.0);
    state.playback.title = mpv
        .get_property::<String>("media-title")
        .unwrap_or_else(|_| "...".to_string());
}

fn build_snapshot(
    state: &AppState,
    cover_bytes: &Option<Vec<u8>>,
    cover_path: &str,
) -> GuiSnapshot {
    let library_is_root = state.library.current_dir == state.library.root_dir;
    let mut library_entries = Vec::new();
    if !library_is_root {
        library_entries.push(LibraryEntryDto::Folder {
            name: "..".to_string(),
        });
    }
    for (_, entry) in state.library.visible_entries() {
        library_entries.push(match entry {
            LibraryEntry::Folder(name) => LibraryEntryDto::Folder {
                name: name.clone(),
            },
            LibraryEntry::Track(track) => LibraryEntryDto::Track {
                title: track.title.clone(),
                path: track.path.clone(),
            },
        });
    }

    let cover_base64 = cover_bytes.as_ref().and_then(|data| {
        if cover_path.is_empty() {
            None
        } else {
            Some(B64.encode(data))
        }
    });

    GuiSnapshot {
        tab: tab_name(state.active_tab).to_string(),
        queue: state
            .queue
            .iter()
            .map(|t| TrackDto {
                title: t.title.clone(),
                path: t.path.clone(),
            })
            .collect(),
        queue_cursor: state.queue_cursor,
        current_track_idx: state.current_track_idx,
        library_path: state.library.displayed_path(),
        library_entries,
        library_selected: state.library.selected_idx,
        library_is_root,
        library_filter: state.library.filter.clone(),
        playlists: state.playlists.playlists.clone(),
        playlist_tracks: state
            .playlists
            .current_tracks
            .iter()
            .map(|t| TrackDto {
                title: t.title.clone(),
                path: t.path.clone(),
            })
            .collect(),
        playlist_viewing: !state.playlists.current_tracks.is_empty(),
        playlist_selected: state.playlists.selected_idx,
        playback: PlaybackDto {
            title: state.playback.title.clone(),
            time: state.playback.time,
            duration: state.playback.duration,
            paused: state.playback.paused,
            muted: state.playback.muted,
            volume: state.playback.volume,
            speed: state.playback.speed,
            is_loop: state.playback.is_loop,
        },
        status_msg: state.status_msg.clone(),
        cover_base64,
    }
}

fn tab_name(tab: Tab) -> &'static str {
    match tab {
        Tab::Queue => "queue",
        Tab::Library => "library",
        Tab::Playlists => "playlists",
        Tab::Stats => "stats",
    }
}

pub fn tab_from_str(s: &str) -> Tab {
    match s {
        "library" => Tab::Library,
        "playlists" => Tab::Playlists,
        "stats" => Tab::Stats,
        _ => Tab::Queue,
    }
}

fn track_title_from_path(input: &str) -> String {
    if input.starts_with("http") {
        input.to_string()
    } else {
        std::path::Path::new(input)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    }
}