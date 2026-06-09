#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::sync::Arc;

use clap::Parser;
use firemusic::core::resolve_music_dir;
use firemusic::gui::GuiRuntime;
use tauri::{Emitter, Manager, RunEvent};

#[derive(Parser, Debug)]
#[command(name = "firemusic-gui", about = "Firemusic desktop GUI")]
struct GuiArgs {
    #[arg(long = "loop", default_value_t = false)]
    loop_mode: bool,

    #[arg(long, default_value_t = 1.0)]
    speed: f64,

    #[arg(long, default_value_t = 100.0)]
    volume: f64,

    #[arg(long = "crossfade", default_value_t = 0.0)]
    crossfade: f64,

    #[arg(long = "music-dir", default_value = "")]
    music_dir: String,

    #[arg(num_args = 0..)]
    inputs: Vec<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = GuiArgs::parse();
    let music_dir = resolve_music_dir(&args.music_dir)?;

    let runtime = Arc::new(GuiRuntime::start(
        music_dir,
        args.inputs,
        args.crossfade,
        args.loop_mode,
        args.volume,
        args.speed,
    )?);

    let runtime_setup = Arc::clone(&runtime);

    tauri::Builder::default()
        .manage(runtime)
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::set_tab,
            commands::library_select,
            commands::library_enter,
            commands::library_back,
            commands::library_rescan,
            commands::library_filter,
            commands::library_add_selected,
            commands::queue_select,
            commands::queue_play,
            commands::queue_remove,
            commands::toggle_pause,
            commands::seek,
            commands::set_volume,
            commands::toggle_mute,
            commands::toggle_loop,
            commands::playlist_select,
            commands::playlist_load,
            commands::playlist_back,
            commands::playlist_refresh,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let rt = Arc::clone(&runtime_setup);
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    let snap = rt.snapshot();
                    let _ = handle.emit("state", &snap);
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .run(|app, event| {
            if let RunEvent::Exit = event {
                if let Some(rt) = app.try_state::<Arc<GuiRuntime>>() {
                    rt.shutdown();
                }
            }
        });

    Ok(())
}