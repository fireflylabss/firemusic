mod state;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use libmpv2::{events::Event as MEvent, Mpv};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::io::Write;
use std::time::Duration;

use state::{AppState, EQ_PRESETS, EQ_PRESET_NAMES, Focus, LibraryEntry, PlaylistManager, Tab, Track};
use ui::render;

fn cleanup_kitty_images(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if supports_graphics_protocol() {
        let _ = terminal.backend_mut().write_all(b"\x1b_Ga=d,d=I;\x1b\\");
        let _ = terminal.backend_mut().flush();
    }
}

fn supports_graphics_protocol() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    
    // Check for Kitty terminal
    if term == "xterm-kitty" || term_program == "kitty" {
        return true;
    }
    
    // Check for other terminals with graphics support
    // WezTerm, iTerm2, etc. could be added here
    
    false
}

pub fn run_tui(
    inputs: Vec<String>, crossfade_duration: f64, is_loop: bool,
    volume: f64, speed: f64, music_dir: &str,
) -> Result<()> {
    // Set up panic handler to ensure terminal cleanup on crashes
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        let _ = std::io::stdout().flush();
        // Call original panic handler
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Delete any lingering kitty images
    cleanup_kitty_images(&mut terminal);

    let mut mpv = Mpv::new().map_err(|e| anyhow::anyhow!("mpv init error: {:?}", e))?;
    let mut app_state = AppState::new(crossfade_duration, is_loop, music_dir);

    if mpv.set_property("video", "no").is_err() {
        app_state.set_message("warning: failed to disable video".to_string());
    }
    if mpv.set_property("volume", volume).is_err() {
        app_state.set_message("warning: failed to set volume".to_string());
    }
    if mpv.set_property("speed", speed).is_err() {
        app_state.set_message("warning: failed to set speed".to_string());
    }
    if mpv.set_property("ytdl", "yes").is_err() {
        app_state.set_message("warning: failed to enable ytdl".to_string());
    }
    if mpv.set_property("ytdl-format", "bestaudio/best").is_err() {
        app_state.set_message("warning: failed to set ytdl format".to_string());
    }
    if crossfade_duration > 0.0 && mpv.set_property("audio-fade", crossfade_duration).is_err() {
        app_state.set_message("warning: failed to set crossfade".to_string());
    }
    if is_loop && mpv.set_property("loop-file", "inf").is_err() {
        app_state.set_message("warning: failed to enable loop".to_string());
    }

    for (i, input) in inputs.iter().enumerate() {
        let mode = if i == 0 { "replace" } else { "append" };
        if mpv.command("loadfile", &[input, mode]).is_err() {
            app_state.set_message(format!("warning: failed to load {}", input));
        }
    }

    app_state.playback.volume = volume;
    app_state.playback.speed = speed;

    for input in &inputs {
        let title = if input.starts_with("http") { input.clone() }
        else { std::path::Path::new(input).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| input.clone()) };
        app_state.queue.push(Track { title, path: input.clone(), duration: 0.0, artist: None, album: None });
    }

    // Try loading cover for first track
    try_load_cover(&mut app_state);

    let res = tui_loop(&mut terminal, &mut mpv, &mut app_state);

    // Cleanup kitty images
    cleanup_kitty_images(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn tui_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mpv: &mut Mpv, state: &mut AppState) -> Result<()> {
    let mut cover_loaded = false;
    let mut last_time = state.playback.time;
    let mut needs_redraw = true;

    loop {
        state.clear_old_message();

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

        // Load cover art once when track starts (after a delay for MPV to get title)
        if !cover_loaded {
            let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
            if state.queue.len() > state.current_track_idx && time > 0.1 {
                try_load_cover(state);
                cover_loaded = true;
                needs_redraw = true;
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
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { break; }
                match handle_key(key, mpv, state) {
                    Ok(true) => { needs_redraw = true; }
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

fn clear_cover_art(state: &mut AppState) {
    // Delete image from Kitty terminal if present
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

fn try_load_cover(state: &mut AppState) {
    if let Some(track) = state.queue.get(state.current_track_idx) {
        let track_path = track.path.clone();
        if track_path == state.playback.last_cover_path { return; }
        if track_path.starts_with("http") { return; }

        state.loading = true;
        state.loading_msg = "loading cover".to_string();

        const MAX_COVER_SIZE: usize = 8 * 1024 * 1024; // 8MB limit

        if let Some(data) = extract_cover_art(&track_path) {
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

        state.loading = false;
        state.loading_msg.clear();
    }
}

fn extract_cover_art(path: &str) -> Option<Vec<u8>> {
    use std::process::Command;
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i", path, "-an", "-vcodec", "png", "-f", "image2pipe", "-vframes", "1", "-"])
        .output().ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        Some(output.stdout)
    } else {
        None
    }
}

fn png_dimensions(data: &[u8]) -> (u32, u32) {
    if data.len() < 24 { return (0, 0); }
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
    state.playback.title = mpv.get_property::<String>("media-title").unwrap_or_else(|_| "...".to_string());
    if let Some(track) = state.queue.get_mut(state.current_track_idx) {
        if !state.playback.title.is_empty() && state.playback.title != "..." { track.title = state.playback.title.clone(); }
    }
}

fn handle_key(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) -> Result<bool> {
    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc { return Ok(false); }

    if key.code == KeyCode::Tab {
        state.focus = match state.focus {
            Focus::List => Focus::NowPlaying,
            Focus::NowPlaying => Focus::Player,
            Focus::Player => Focus::List,
        };
        state.set_message(format!("focus: {}", state.focus.label()));
        return Ok(true);
    }

    match key.code {
        KeyCode::F(1) => { state.active_tab = Tab::Queue; if state.active_tab == Tab::Playlists { state.playlists.current_tracks.clear(); } return Ok(true); }
        KeyCode::F(2) => { state.active_tab = Tab::Library; return Ok(true); }
        KeyCode::F(3) => { state.active_tab = Tab::Playlists; state.playlists.refresh(); state.playlists.current_tracks.clear(); return Ok(true); }
        KeyCode::F(4) => { state.active_tab = Tab::Stats; return Ok(true); }
        KeyCode::Char('?') => { state.show_help_popup = !state.show_help_popup; return Ok(true); }
        _ => {}
    }

    if state.focus == Focus::Player {
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
                if mpv.set_property("loop-file", if state.playback.is_loop { "inf" } else { "no" }).is_err() {
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
                if mpv.set_property("speed", 1.0).is_err() || mpv.set_property("pitch", 1.0).is_err() {
                    state.set_message("warning: failed to reset speed/pitch".to_string());
                }
                return Ok(true);
            }
            KeyCode::Char(c) if c.is_digit(10) && c != '0' => {
                let pct = c.to_digit(10).unwrap() * 10;
                if mpv.command("seek", &[&pct.to_string(), "absolute-percent"]).is_err() {
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
            KeyCode::Char('e') => { cycle_eq(mpv, state); return Ok(true); }
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

fn cycle_eq(mpv: &Mpv, state: &mut AppState) {
    state.eq_preset = (state.eq_preset + 1) % EQ_PRESETS.len();
    let preset = EQ_PRESETS.get(state.eq_preset).unwrap_or(&"");
    mpv.set_property("af", *preset).ok();
    let name = EQ_PRESET_NAMES.get(state.eq_preset).unwrap_or(&"unknown");
    state.set_message(format!("EQ: {}", name));
}

fn handle_queue_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { if state.queue_cursor > 0 { state.queue_cursor -= 1; } else if !state.queue.is_empty() { state.queue_cursor = state.queue.len().saturating_sub(1); } }
        KeyCode::Down | KeyCode::Char('j') => { if state.queue_cursor + 1 < state.queue.len() { state.queue_cursor += 1; } else { state.queue_cursor = 0; } }
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
            if state.queue.is_empty() { return; }
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
            if mpv.set_property("loop-file", if state.playback.is_loop { "inf" } else { "no" }).is_err() {
                state.set_message("warning: failed to toggle loop".to_string());
            }
        }
        KeyCode::Char('e') => { cycle_eq(mpv, state); }
        _ => {}
    }
}

fn handle_library_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    let is_root = state.library.current_dir == state.library.root_dir;
    let offset = if is_root { 0 } else { 1 };
    let max_idx = state.library.entries.len().saturating_sub(1) + offset;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { if state.library.selected_idx > 0 { state.library.selected_idx -= 1; } else if max_idx > 0 { state.library.selected_idx = max_idx; } }
        KeyCode::Down | KeyCode::Char('j') => { if state.library.selected_idx < max_idx { state.library.selected_idx += 1; } else { state.library.selected_idx = 0; } }
        KeyCode::Enter | KeyCode::Right => {
            if !is_root && state.library.selected_idx == 0 { state.library.go_to_parent(); return; }
            let eidx = if is_root { state.library.selected_idx } else { state.library.selected_idx.saturating_sub(1) };
            if let Some(entry) = state.library.entries.get(eidx).cloned() {
                match entry {
                    LibraryEntry::Folder(_) => { state.library.selected_idx = eidx; state.library.enter_folder(); }
                    LibraryEntry::Track(track) => {
                        state.queue.push(track.clone());
                        if mpv.command("loadfile", &[&track.path, "append-play"]).is_err() {
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
            let eidx = if is_root { state.library.selected_idx } else { state.library.selected_idx.saturating_sub(1) };
            if let Some(LibraryEntry::Track(track)) = state.library.entries.get(eidx) {
                state.queue.push(track.clone());
                if mpv.command("loadfile", &[&track.path, "append-play"]).is_err() {
                    state.set_message("warning: failed to add track".to_string());
                } else {
                    state.set_message(format!("added: {}", track.title));
                }
            }
        }
        KeyCode::Char('c') => {
            disable_raw_mode().ok();
            let new_path: String = dialoguer::Input::new().with_prompt("library directory").default(state.library.current_dir.to_string_lossy().to_string()).interact_text().unwrap_or_default();
            enable_raw_mode().ok();
            if !new_path.is_empty() {
                let p = std::path::PathBuf::from(&new_path);
                if p.exists() {
                    state.library.change_root(p);
                    state.set_message(format!("library: {}", new_path));
                } else {
                    state.set_message("error: path does not exist".to_string());
                }
            }
        }
        KeyCode::Char('r') => {
            state.library.scan_current_dir_with_callback(|loading, msg| {
                state.loading = loading;
                state.loading_msg = msg.to_string();
            });
            state.set_message("library scanned".to_string());
        }
        KeyCode::Char('e') => { cycle_eq(mpv, state); }
        _ => {}
    }
}

fn handle_playlist_keys(key: event::KeyEvent, mpv: &Mpv, state: &mut AppState) {
    let max = state.playlists.total_items();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { if state.playlists.selected_idx > 0 { state.playlists.selected_idx -= 1; } else if max > 0 { state.playlists.selected_idx = max.saturating_sub(1); } }
        KeyCode::Down | KeyCode::Char('j') => { if state.playlists.selected_idx + 1 < max { state.playlists.selected_idx += 1; } else { state.playlists.selected_idx = 0; } }
        KeyCode::Enter => {
            if state.playlists.current_tracks.is_empty() {
                if state.playlists.selected_idx < state.playlists.playlists.len() {
                    let name = state.playlists.playlists.get(state.playlists.selected_idx).cloned();
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
            disable_raw_mode().ok();
            let name: String = dialoguer::Input::new().with_prompt("playlist name").interact_text().unwrap_or_default();
            enable_raw_mode().ok();
            if !name.is_empty() {
                if state.playlists.save_playlist(&name, &[]).is_ok() {
                    state.playlists.refresh();
                    state.set_message(format!("created: {}", name));
                } else {
                    state.set_message("error: failed to create playlist".to_string());
                }
            }
        }
        KeyCode::Char('s') => {
            if !state.queue.is_empty() {
                disable_raw_mode().ok();
                let name: String = dialoguer::Input::new().with_prompt("save playlist as").interact_text().unwrap_or_default();
                enable_raw_mode().ok();
                if !name.is_empty() {
                    if state.playlists.save_playlist(&name, &state.queue).is_ok() {
                        state.playlists.refresh();
                        state.set_message(format!("saved: {}", name));
                    } else {
                        state.set_message("error: failed to save playlist".to_string());
                    }
                }
            } else {
                state.set_message("error: queue is empty".to_string());
            }
        }
        KeyCode::Char('d') => {
            if state.playlists.current_tracks.is_empty() {
                if state.playlists.selected_idx < state.playlists.playlists.len() {
                    let name = state.playlists.playlists.get(state.playlists.selected_idx).cloned();
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
            if let Some(track) = state.playlists.current_tracks.get(state.playlists.selected_idx) {
                state.queue.push(track.clone());
                if mpv.command("loadfile", &[&track.path, "append-play"]).is_err() {
                    state.set_message("warning: failed to add track".to_string());
                } else {
                    state.set_message(format!("added: {}", track.title));
                }
            }
        }
        KeyCode::Char('e') => { cycle_eq(mpv, state); }
        _ => {}
    }
}
