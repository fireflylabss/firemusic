mod app;
mod event_loop;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::io::Write;
use std::path::PathBuf;

use crate::core::{create_player, load_inputs, validate_playback_inputs, MpvConfig};

use app::{AppState, Track};
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
    term == "xterm-kitty" || term_program == "kitty"
}

pub fn run_tui(
    inputs: Vec<String>,
    crossfade_duration: f64,
    is_loop: bool,
    volume: f64,
    speed: f64,
    music_dir: PathBuf,
) -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        let _ = std::io::stdout().flush();
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    cleanup_kitty_images(&mut terminal);

    let config = MpvConfig::for_cli(volume, speed, is_loop, crossfade_duration);
    let mut mpv = create_player(&config)?;
    let mut app_state = AppState::new(crossfade_duration, is_loop, music_dir);

    let validated_inputs = if inputs.is_empty() {
        Vec::new()
    } else {
        match validate_playback_inputs(&inputs) {
            Ok(validated) => {
                if let Err(err) = load_inputs(&mpv, &validated) {
                    app_state.set_message(format!("warning: {}", err));
                }
                validated
            }
            Err(err) => {
                app_state.set_message(format!("warning: {}", err));
                Vec::new()
            }
        }
    };

    app_state.playback.volume = volume;
    app_state.playback.speed = speed;

    for input in &validated_inputs {
        let title = if input.starts_with("http") {
            input.clone()
        } else {
            std::path::Path::new(input)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| input.clone())
        };
        app_state.queue.push(Track {
            title,
            path: input.clone(),
            duration: 0.0,
            artist: None,
            album: None,
        });
    }

    terminal.draw(|f| render(f, &app_state))?;

    let res = event_loop::run(&mut terminal, &mut mpv, &mut app_state);

    cleanup_kitty_images(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}