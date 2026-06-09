use anyhow::Result;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
};
use libmpv2::{Mpv, events::Event as MEvent};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

use super::app::{
    AppState, EQ_PRESET_NAMES, EQ_PRESETS, Focus, InputAction, LibraryEntry, PlaylistManager, Tab,
};
use super::ui::render;
use crate::core::resolve_music_dir;

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mpv: &mut Mpv,
    state: &mut AppState,
) -> Result<()> {
    let mut cover_loaded = false;
    let mut cover_loading_path: Option<String> = None;
    let (cover_tx, cover_rx) = mpsc::channel::<(String, Option<Vec<u8>>)>();
    let mut last_track_path: Option<String> = state
        .queue
        .get(state.current_track_idx)
        .map(|track| track.path.clone());
    let mut last_time = state.playback.time;
    let mut needs_redraw = true;

    loop {
        state.clear_old_message();

        let current_track_path = state
            .queue
            .get(state.current_track_idx)
            .map(|track| track.path.clone());
        if current_track_path != last_track_path {
            last_track_path = current_track_path;
            cover_loaded = false;
            cover_loading_path = None;
        }

        if let Some(event_result) = mpv.wait_event(0.0) {
            match event_result.map_err(|e| anyhow::anyhow!("mpv error: {:?}", e))? {
                MEvent::EndFile(_reason) => {
                    if !state.playback.is_loop {
                        let remaining: i64 = mpv.get_property("playlist-count").unwrap_or(0);
                        let pos: i64 = mpv.get_property("playlist-pos").unwrap_or(0);
                        if pos + 1 >= remaining || pos < 0 {
                            if state.current_track_idx + 1 < state.queue.len() {
                                state.current_track_idx += 1;
                                state.queue_cursor = state.current_track_idx;
                                let next = &state.queue[state.current_track_idx];
                                mpv.command("loadfile", &[&next.path, "replace"]).ok();
                                state.set_message(format!("now playing: {}", next.title));
                                cover_loaded = false;
                                needs_redraw = true;
                            }
                        }
                    }
                }
                MEvent::Shutdown => break,
                _ => {}
            }
        }

        while let Ok((path, data)) = cover_rx.try_recv() {
            if state
                .queue
                .get(state.current_track_idx)
                .map(|track| track.path.as_str())
                == Some(path.as_str())
            {
                apply_cover_result(state, path, data);
                cover_loaded = true;
                needs_redraw = true;
            }
            cover_loading_path = None;
            state.loading = false;
            state.loading_msg.clear();
        }

        // Load cover art once when track starts (after a delay for MPV to get title)
        if !cover_loaded {
            let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
            if state.queue.len() > state.current_track_idx && time > 0.1 {
                if let Some(path) =
                    start_cover_load(state, &cover_tx, cover_loading_path.as_deref())
                {
                    cover_loading_path = Some(path);
                    needs_redraw = true;
                } else if state
                    .queue
                    .get(state.current_track_idx)
                    .map(|track| track.path.starts_with("http"))
                    .unwrap_or(false)
                {
                    cover_loaded = true;
                }
            }
        }

        update_playback_state(mpv, state);

        // Check if playback time changed significantly (for progress bar updates)
        if (state.playback.time - last_time).abs() > 0.5 {
            last_time = state.playback.time;
            needs_redraw = true;
        }

        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }
                match handle_key(key, mpv, state) {
                    Ok(true) => {
                        needs_redraw = true;
                    }
                    Ok(false) => break,
                    Err(_) => {}
                }
            }
        }

        if needs_redraw {
            terminal.draw(|f| render(f, state))?;
            needs_redraw = false;
        }

        std::thread::sleep(Duration::from_millis(16)); // ~60fps
    }
    Ok(())
}

fn supports_graphics_protocol() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    term == "xterm-kitty" || term_program == "kitty"
}

fn clear_cover_art(state: &mut AppState) {
    if supports_graphics_protocol() && state.playback.cover_id > 0 {
        let del = format!("\x1b_Ga=d,i={};\x1b\\", state.playback.cover_id);
        let _ = std::io::stdout().write_all(del.as_bytes());
        let _ = std::io::stdout().flush();
    }

    // Clear cover art data and reset state
    state.playback.cover_art = None;
    state.playback.cover_id = 0;
    state.playback.cover_w = 0;
    state.playback.cover_h = 0;
    state.playback.last_cover_path.clear();
}

fn start_cover_load(
    state: &mut AppState,
    tx: &mpsc::Sender<(String, Option<Vec<u8>>)>,
    loading_path: Option<&str>,
) -> Option<String> {
    if let Some(track) = state.queue.get(state.current_track_idx) {
        let track_path = track.path.clone();
        if track_path == state.playback.last_cover_path {
            return None;
        }
        if track_path.starts_with("http") {
            return None;
        }
        if loading_path == Some(track_path.as_str()) {
            return None;
        }

        state.loading = true;
        state.loading_msg = "loading cover".to_string();
        let tx = tx.clone();
        let thread_path = track_path.clone();
        std::thread::spawn(move || {
            let data = extract_cover_art(&thread_path);
            let _ = tx.send((thread_path, data));
        });
        return Some(track_path);
    }
    None
}

fn apply_cover_result(state: &mut AppState, track_path: String, data: Option<Vec<u8>>) {
    const MAX_COVER_SIZE: usize = 8 * 1024 * 1024; // 8MB limit

    if let Some(data) = data {
        if data.len() > 8 && data.len() <= MAX_COVER_SIZE {
            let (w, h) = png_dimensions(&data);

            // Clear old cover art first to prevent memory leak
            clear_cover_art(state);

            state.playback.cover_id = state.playback.cover_id.wrapping_add(1);
            state.playback.cover_art = Some(data);
            state.playback.cover_w = w;
            state.playback.cover_h = h;
            state.playback.last_cover_path = track_path;
        } else if data.len() > MAX_COVER_SIZE {
            state.set_message("warning: cover art too large, skipping".to_string());
        }
    }
}

fn extract_cover_art(path: &str) -> Option<Vec<u8>> {
    use std::process::Command;
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            path,
            "-an",
            "-vcodec",
            "png",
            "-f",
            "image2pipe",
            "-vframes",
            "1",
            "-",
        ])
        .output()
        .ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        Some(output.stdout)
    } else {
        None
    }
}

fn png_dimensions(data: &[u8]) -> (u32, u32) {
    if data.len() < 24 {
        return (0, 0);
    }
    let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    (w, h)
}

fn update_playback_state(mpv: &Mpv, state: &mut AppState) {
    state.playback.time = mpv.get_property::<f64>("time-pos").unwrap_or(0.0);
    state.playback.duration = mpv.get_property::<f64>("duration").unwrap_or(0.0);
    state.playback.paused = mpv.get_property::<bool>("pause").unwrap_or(false);
    state.playback.muted = mpv.get_property::<bool>("mute").unwrap_or(false);
    state.playback.volume = mpv.get_property::<f64>("volume").unwrap_or(100.0);
    state.playback.speed = mpv.get_property::<f64>("speed").unwrap_or(1.0);
    state.playback.pitch = mpv.get_property::<f64>("pitch").unwrap_or(1.0);
    state.playback.bitrate_kbps = mpv.get_property::<f64>("audio-bitrate").unwrap_or(0.0) / 1000.0;
    state.playback.title = mpv
        .get_property::<String>("media-title")
        .unwrap_or_else(|_| "...".to_string());
    if let Some(track) = state.queue.get_mut(state.current_track_idx) {
        if !state.playback.title.is_empty() && state.playback.title != "..." {
            track.title = state.playback.title.clone();
        }
    }
}

fn handle_key(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) -> Result<bool> {
    if state.input_mode.is_some() {
        handle_input_key(key, mpv, state);
        return Ok(true);
    }

    if key.code == KeyCode::Char('q') {
        return Ok(false);
    }

    if key.code == KeyCode::Esc {
        if state.show_help_popup {
            state.show_help_popup = false;
            return Ok(true);
        }
        if state.active_tab == Tab::Playlists && !state.playlists.current_tracks.is_empty() {
            state.playlists.current_tracks.clear();
            state.playlists.refresh();
            state.playlists.selected_idx = 0;
            state.set_message("back to playlists".to_string());
            return Ok(true);
        }
        if state.active_tab == Tab::Library && !state.library.filter.is_empty() {
            state.library.filter.clear();
            state.library.selected_idx = 0;
            state.set_message("filter cleared".to_string());
            return Ok(true);
        }
        return Ok(false);
    }

    if key.code == KeyCode::Tab {
        state.focus = match state.focus {
            Focus::List => Focus::NowPlaying,
            Focus::NowPlaying => Focus::List,
        };
        state.set_message(format!("focus: {}", state.focus.label()));
        return Ok(true);
    }

    match key.code {
        KeyCode::F(1) => {
            let was_playlists = state.active_tab == Tab::Playlists;
            state.active_tab = Tab::Queue;
            if was_playlists {
                state.playlists.current_tracks.clear();
            }
            return Ok(true);
        }
        KeyCode::F(2) => {
            if state.active_tab == Tab::Playlists {
                state.playlists.current_tracks.clear();
            }
            state.active_tab = Tab::Library;
            return Ok(true);
        }
        KeyCode::F(3) => {
            state.active_tab = Tab::Playlists;
            state.playlists.refresh();
            state.playlists.current_tracks.clear();
            return Ok(true);
        }
        KeyCode::F(4) => {
            if state.active_tab == Tab::Playlists {
                state.playlists.current_tracks.clear();
            }
            state.active_tab = Tab::Stats;
            return Ok(true);
        }
        KeyCode::Char('?') => {
            state.show_help_popup = !state.show_help_popup;
            return Ok(true);
        }
        _ => {}
    }

    if state.focus == Focus::NowPlaying {
        match key.code {
            KeyCode::Char(' ') => {
                let p: bool = mpv.get_property("pause").unwrap_or(false);
                if mpv.set_property("pause", !p).is_err() {
                    state.set_message("warning: failed to toggle pause".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('m') => {
                let m: bool = mpv.get_property("mute").unwrap_or(false);
                if mpv.set_property("mute", !m).is_err() {
                    state.set_message("warning: failed to toggle mute".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('l') => {
                state.playback.is_loop = !state.playback.is_loop;
                if mpv
                    .set_property(
                        "loop-file",
                        if state.playback.is_loop { "inf" } else { "no" },
                    )
                    .is_err()
                {
                    state.set_message("warning: failed to toggle loop".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char(',') => {
                let c: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                if mpv.set_property("pitch", (c - 0.05).max(0.5)).is_err() {
                    state.set_message("warning: failed to set pitch".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('.') => {
                let c: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                if mpv.set_property("pitch", (c + 0.05).min(2.0)).is_err() {
                    state.set_message("warning: failed to set pitch".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('{') => {
                if mpv.command("seek", &["-60", "relative"]).is_err() {
                    state.set_message("warning: seek failed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('}') => {
                if mpv.command("seek", &["60", "relative"]).is_err() {
                    state.set_message("warning: seek failed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Right => {
                if mpv.command("seek", &["5", "relative"]).is_err() {
                    state.set_message("warning: seek failed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if mpv.command("seek", &["-5", "relative"]).is_err() {
                    state.set_message("warning: seek failed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Up => {
                let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                if mpv.set_property("volume", (v + 5.0).min(100.0)).is_err() {
                    state.set_message("warning: failed to set volume".to_string());
                }
                return Ok(true);
            }
            KeyCode::Down => {
                let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                if mpv.set_property("volume", (v - 5.0).max(0.0)).is_err() {
                    state.set_message("warning: failed to set volume".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('0') => {
                if mpv.set_property("speed", 1.0).is_err()
                    || mpv.set_property("pitch", 1.0).is_err()
                {
                    state.set_message("warning: failed to reset speed/pitch".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char(c) if c.is_digit(10) && c != '0' => {
                let pct = c.to_digit(10).unwrap() * 10;
                if mpv
                    .command("seek", &[&pct.to_string(), "absolute-percent"])
                    .is_err()
                {
                    state.set_message("warning: seek failed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                if mpv.set_property("speed", (s + 0.1).min(10.0)).is_err() {
                    state.set_message("warning: failed to set speed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                if mpv.set_property("speed", (s - 0.1).max(0.1)).is_err() {
                    state.set_message("warning: failed to set speed".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char('e') => {
                cycle_eq(mpv, state);
                return Ok(true);
            }
            _ => {}
        }
    }

    // Only handle tab-specific keys when focus is on the list
    if state.focus == Focus::List {
        match state.active_tab {
            Tab::Queue => handle_queue_keys(key, mpv, state),
            Tab::Library => handle_library_keys(key, mpv, state),
            Tab::Playlists => handle_playlist_keys(key, mpv, state),
            Tab::Stats => {} // Stats tab is read-only
        }
    }
    Ok(true)
}

fn handle_input_key(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    match key.code {
        KeyCode::Esc => {
            state.input_mode = None;
            state.set_message("cancelled".to_string());
        }
        KeyCode::Backspace => {
            if let Some(input) = &mut state.input_mode {
                input.value.pop();
            }
        }
        KeyCode::Enter => {
            if let Some(input) = state.input_mode.take() {
                apply_input(input.action, input.value, mpv, state);
            }
        }
        KeyCode::Char(c) => {
            if let Some(input) = &mut state.input_mode {
                input.value.push(c);
            }
        }
        _ => {}
    }
}

fn apply_input(action: InputAction, value: String, mpv: &Mpv, state: &mut AppState) {
    let value = value.trim().to_string();
    match action {
        InputAction::ChangeLibraryRoot => {
            if value.is_empty() {
                state.set_message("cancelled".to_string());
                return;
            }
            match resolve_music_dir(&value) {
                Ok(p) => {
                    state.library.change_root(p.clone());
                    state.set_message(format!("library: {}", p.display()));
                }
                Err(err) => state.set_message(format!("error: {}", err)),
            }
        }
        InputAction::NewPlaylist => {
            if value.is_empty() {
                state.set_message("cancelled".to_string());
            } else if state.playlists.save_playlist(&value, &[]).is_ok() {
                state.playlists.refresh();
                state.set_message(format!("created: {}", value));
            } else {
                state.set_message("error: failed to create playlist".to_string());
            }
        }
        InputAction::SavePlaylist => {
            if value.is_empty() {
                state.set_message("cancelled".to_string());
            } else if state.playlists.save_playlist(&value, &state.queue).is_ok() {
                state.playlists.refresh();
                state.set_message(format!("saved: {}", value));
            } else {
                state.set_message("error: failed to save playlist".to_string());
            }
        }
        InputAction::FilterLibrary => {
            state.library.filter = value;
            state.library.selected_idx = 0;
            let count = state.library.visible_entries().len();
            state.set_message(format!(
                "filter: {} result{}",
                count,
                if count == 1 { "" } else { "s" }
            ));
        }
    }
    update_playback_state(mpv, state);
}

fn cycle_eq(mpv: &Mpv, state: &mut AppState) {
    state.eq_preset = (state.eq_preset + 1) % EQ_PRESETS.len();
    let preset = EQ_PRESETS.get(state.eq_preset).unwrap_or(&"");
    mpv.set_property("af", *preset).ok();
    let name = EQ_PRESET_NAMES.get(state.eq_preset).unwrap_or(&"unknown");
    state.set_message(format!("EQ: {}", name));
}

fn handle_queue_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.queue_cursor > 0 {
                state.queue_cursor -= 1;
            } else if !state.queue.is_empty() {
                state.queue_cursor = state.queue.len().saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.queue_cursor + 1 < state.queue.len() {
                state.queue_cursor += 1;
            } else {
                state.queue_cursor = 0;
            }
        }
        KeyCode::Enter => {
            if let Some(track) = state.queue.get(state.queue_cursor) {
                let track_path = track.path.clone();
                let track_title = track.title.clone();
                state.current_track_idx = state.queue_cursor;

                // Clear cover art when changing tracks to prevent memory leak
                clear_cover_art(state);

                if mpv.command("loadfile", &[&track_path, "replace"]).is_err() {
                    state.set_message("warning: failed to load track".to_string());
                } else {
                    state.set_message(format!("playing: {}", track_title));
                }
            }
        }
        KeyCode::Char('d') => {
            if state.queue.is_empty() {
                return;
            }
            let idx = state.queue_cursor;
            let was_playing = idx == state.current_track_idx;

            // Clear cover art to prevent memory leak when removing tracks
            clear_cover_art(state);

            // Update current_track_idx if removing a track before it
            if idx < state.current_track_idx {
                state.current_track_idx = state.current_track_idx.saturating_sub(1);
            }

            state.queue.remove(idx);

            if state.queue.is_empty() {
                state.current_track_idx = 0;
                state.queue_cursor = 0;
                state.set_message("queue empty".to_string());
                return;
            }

            // Adjust cursor position if needed
            if state.queue_cursor >= state.queue.len() {
                state.queue_cursor = state.queue.len().saturating_sub(1);
            }

            // Ensure current_track_idx is valid
            if state.current_track_idx >= state.queue.len() {
                state.current_track_idx = 0;
            }

            if was_playing {
                if let Some(t) = state.queue.get(state.current_track_idx) {
                    if mpv.command("loadfile", &[&t.path, "replace"]).is_err() {
                        state.set_message("warning: failed to load next track".to_string());
                    } else {
                        state.set_message("removed, playing next".to_string());
                    }
                }
            } else {
                state.set_message("removed".to_string());
            }
        }
        KeyCode::Char(' ') => {
            let p: bool = mpv.get_property("pause").unwrap_or(false);
            if mpv.set_property("pause", !p).is_err() {
                state.set_message("warning: failed to toggle pause".to_string());
            }
        }
        KeyCode::Char('m') => {
            let m: bool = mpv.get_property("mute").unwrap_or(false);
            if mpv.set_property("mute", !m).is_err() {
                state.set_message("warning: failed to toggle mute".to_string());
            }
        }
        KeyCode::Char('l') => {
            state.playback.is_loop = !state.playback.is_loop;
            if mpv
                .set_property(
                    "loop-file",
                    if state.playback.is_loop { "inf" } else { "no" },
                )
                .is_err()
            {
                state.set_message("warning: failed to toggle loop".to_string());
            }
        }
        KeyCode::Char('e') => {
            cycle_eq(mpv, state);
        }
        _ => {}
    }
}

fn handle_library_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    let is_root = state.library.current_dir == state.library.root_dir;
    let offset = if is_root { 0 } else { 1 };
    let visible = state.library.visible_entries();
    let max_idx = visible.len().saturating_sub(1) + offset;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.library.selected_idx > 0 {
                state.library.selected_idx -= 1;
            } else if max_idx > 0 {
                state.library.selected_idx = max_idx;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.library.selected_idx < max_idx {
                state.library.selected_idx += 1;
            } else {
                state.library.selected_idx = 0;
            }
        }
        KeyCode::Enter | KeyCode::Right => {
            if !is_root && state.library.selected_idx == 0 {
                state.library.go_to_parent();
                return;
            }
            let vidx = if is_root {
                state.library.selected_idx
            } else {
                state.library.selected_idx.saturating_sub(1)
            };
            if let Some((eidx, entry)) = visible
                .get(vidx)
                .map(|(idx, entry)| (*idx, (*entry).clone()))
            {
                match entry {
                    LibraryEntry::Folder(_) => {
                        state.library.selected_idx = eidx;
                        state.library.enter_folder();
                    }
                    LibraryEntry::Track(track) => {
                        state.queue.push(track.clone());
                        if mpv
                            .command("loadfile", &[&track.path, "append-play"])
                            .is_err()
                        {
                            state.set_message("warning: failed to add track".to_string());
                        } else {
                            state.set_message(format!("added: {}", track.title));
                        }
                    }
                }
            }
        }
        KeyCode::Left | KeyCode::Backspace => state.library.go_to_parent(),
        KeyCode::Char('a') => {
            let vidx = if is_root {
                state.library.selected_idx
            } else {
                state.library.selected_idx.saturating_sub(1)
            };
            if let Some((_, LibraryEntry::Track(track))) = visible.get(vidx) {
                state.queue.push(track.clone());
                if mpv
                    .command("loadfile", &[&track.path, "append-play"])
                    .is_err()
                {
                    state.set_message("warning: failed to add track".to_string());
                } else {
                    state.set_message(format!("added: {}", track.title));
                }
            }
        }
        KeyCode::Char('c') => {
            state.start_input(
                InputAction::ChangeLibraryRoot,
                "library directory",
                state.library.current_dir.to_string_lossy().to_string(),
            );
        }
        KeyCode::Char('/') => {
            state.start_input(
                InputAction::FilterLibrary,
                "filter library",
                state.library.filter.clone(),
            );
        }
        KeyCode::Char('r') => {
            state
                .library
                .scan_current_dir_with_callback(|loading, msg| {
                    state.loading = loading;
                    state.loading_msg = msg.to_string();
                });
            state.set_message("library scanned".to_string());
        }
        KeyCode::Char('e') => {
            cycle_eq(mpv, state);
        }
        _ => {}
    }
}

fn handle_playlist_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    let max = state.playlists.total_items();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.playlists.selected_idx > 0 {
                state.playlists.selected_idx -= 1;
            } else if max > 0 {
                state.playlists.selected_idx = max.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.playlists.selected_idx + 1 < max {
                state.playlists.selected_idx += 1;
            } else {
                state.playlists.selected_idx = 0;
            }
        }
        KeyCode::Enter => {
            if state.playlists.current_tracks.is_empty() {
                if state.playlists.selected_idx < state.playlists.playlists.len() {
                    let name = state
                        .playlists
                        .playlists
                        .get(state.playlists.selected_idx)
                        .cloned();
                    if let Some(name) = name {
                        state.playlists.load_playlist(&name);
                        state.set_message(format!("loaded: {}", name));
                    }
                }
            }
        }
        KeyCode::Backspace => {
            state.playlists.current_tracks.clear();
            state.playlists.refresh();
            state.playlists.selected_idx = 0;
            state.set_message("back to playlists".to_string());
        }
        KeyCode::Char('n') => {
            state.start_input(InputAction::NewPlaylist, "playlist name", "");
        }
        KeyCode::Char('s') => {
            if !state.queue.is_empty() {
                state.start_input(InputAction::SavePlaylist, "save playlist as", "");
            } else {
                state.set_message("error: queue is empty".to_string());
            }
        }
        KeyCode::Char('d') => {
            if state.playlists.current_tracks.is_empty() {
                if state.playlists.selected_idx < state.playlists.playlists.len() {
                    let name = state
                        .playlists
                        .playlists
                        .get(state.playlists.selected_idx)
                        .cloned();
                    if let Some(name) = name {
                        if PlaylistManager::delete_playlist(&name).is_ok() {
                            state.playlists.refresh();
                            state.set_message(format!("deleted: {}", name));
                        } else {
                            state.set_message("error: failed to delete playlist".to_string());
                        }
                    }
                }
            }
        }
        KeyCode::Char('a') => {
            if let Some(track) = state
                .playlists
                .current_tracks
                .get(state.playlists.selected_idx)
            {
                state.queue.push(track.clone());
                if mpv
                    .command("loadfile", &[&track.path, "append-play"])
                    .is_err()
                {
                    state.set_message("warning: failed to add track".to_string());
                } else {
                    state.set_message(format!("added: {}", track.title));
                }
            }
        }
        KeyCode::Char('e') => {
            cycle_eq(mpv, state);
        }
        _ => {}
    }
}
