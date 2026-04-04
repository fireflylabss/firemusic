# 💎 Project Context: firemusic (fire music)

## Project Overview
**firemusic** is a high-performance, minimalist CLI music player built with Rust. It leverages the **MPV** engine via `libmpv2` to provide a sleek and robust playback experience directly in the terminal.

- **Purpose:** Provide a tactical, zero-leak CLI interface for playing local files and web streams, with advanced downloading capabilities.
- **Main Technologies:** Rust, `libmpv2` (MPV 2.0+), `yt-dlp` (for downloads and web playback), `dialoguer` (interactive wizard).
- **Key Features:**
    - Tactical 3-line fixed UI (no terminal clutter).
    - Advanced Download System: Interactive multi-format wizard via `--download`.
    - Web stream support via `ytdl` (YouTube, SoundCloud, etc.).
    - Integrated status indicators and control keys in the UI.
    - Sequential playlist support with automatic exit.

## Building and Running

### Prerequisites
- **libmpv 2.0+** development files must be installed.
- **yt-dlp** must be in the system PATH for web features and downloading.

### Key Commands
- **Automated Install (Unix / macOS):** `curl -sSL https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_unix.sh | bash`
- **Automated Install (Windows):** `powershell -c "irm https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_windows.ps1 | iex"`
- **Build & Install (Manual):** `cargo install --path .`
- **Run:** `msc <inputs...>`
- **Download Wizard:** `msc --download`
- **Preset Download:** `msc --download=audio <url>`

## Uninstallation

### Windows
- Delete the folder: `C:\Users\<User>\.fireflylabs\firemusic`
- (Optional) Remove the folder from your User PATH in Environment Variables.

### Linux / macOS
- Delete the binary: `rm $(which msc)`
- (If installed via cargo): `cargo uninstall firemusic`
- Delete system dependencies (optional): `apt remove libmpv-dev yt-dlp` or `brew uninstall mpv yt-dlp`

## Development Conventions

### UI Rendering (The "Zero-Leak" Engine)
- The UI must always be confined to a **strictly managed 3-line block**.
- Use **absolute cursor positioning** (`MoveToColumn`, `MoveUp/Down`) and `queue!` for atomic updates.
- All track titles and shortcut references must be **truncated** to fit the terminal width exactly.
- **Lowercase aesthetic:** Labels must remain lowercase (e.g., `now playing:`, `volume:`).

### Tactical Styling
- Integrate status information at the start of the line: `[playing | mute]`.
- Enclose all keyboard references in brackets `[]`.
- Use actual arrow symbols (`←`, `→`, `↑`, `↓`) for movement keys.

### Download System
- The interactive wizard uses `dialoguer` for selects and inputs.
- Non-interactive presets (`--download=audio/video`) bypass the wizard for speed.
- Support multi-format selection and independent filenames for different streams.

## Legal
- **License:** Apache License 2.0 (Copyright © 2026-Present Firefly Labs).
