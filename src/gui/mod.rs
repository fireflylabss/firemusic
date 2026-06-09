mod cover;
pub mod runtime;
pub mod snapshot;

use std::path::PathBuf;
use std::process::Command;

pub use runtime::GuiRuntime;
pub use snapshot::GuiSnapshot;

pub fn launch_gui(
    music_dir: PathBuf,
    inputs: Vec<String>,
    crossfade: f64,
    is_loop: bool,
    volume: f64,
    speed: f64,
) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let gui_name = if cfg!(windows) {
        "firemusic-gui.exe"
    } else {
        "firemusic-gui"
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(parent) = exe.parent() {
        candidates.push(parent.join(gui_name));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("src-tauri/target/release").join(gui_name));
        candidates.push(cwd.join("src-tauri/target/debug").join(gui_name));
    }

    for path in candidates {
        if path.exists() {
            let mut cmd = Command::new(&path);
            if is_loop {
                cmd.arg("--loop");
            }
            if (volume - 100.0).abs() > f64::EPSILON {
                cmd.args(["--volume", &volume.to_string()]);
            }
            if (speed - 1.0).abs() > f64::EPSILON {
                cmd.args(["--speed", &speed.to_string()]);
            }
            if crossfade > 0.0 {
                cmd.args(["--crossfade", &crossfade.to_string()]);
            }
            if let Some(dir) = music_dir.to_str() {
                if !dir.is_empty() {
                    cmd.args(["--music-dir", dir]);
                }
            }
            cmd.args(inputs);
            let status = cmd.status()?;
            if status.success() {
                return Ok(());
            }
            anyhow::bail!("firemusic-gui exited with {}", status);
        }
    }
    anyhow::bail!(
        "firemusic-gui not found. Build it with:\n  cd gui && bun install && bun run build\n  cd src-tauri && cargo build --release\n  cargo install --path src-tauri"
    )
}