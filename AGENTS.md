# FireMusic - Agent Guide

## Project Overview

- **Name**: FireMusic
- **Stack**: Rust (edition 2024), `libmpv2`, `yt-dlp`, `crossterm`, `ratatui`
- **Purpose**: High-performance CLI music player for local files and web streams, with discovery, download, and optional TUI
- **Binaries**: `msc` (primary), `firemusic` (alias via `cargo install`)
- **Version**: 0.2.8
- **License**: Apache License 2.0

## Commands

```bash
# Build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run directly
cargo run -- <file-or-url>
cargo run -- -s "query"
cargo run -- --tui
cargo run -- --download

# Install binaries globally (msc + firemusic)
cargo install --path .

# Platform installers (strict dependency check, no auto-install)
./scripts/install_linux.sh
./scripts/install_macos.sh
# Windows: scripts/install_windows.ps1
```

## Post-Task Workflow

After completing any development task, ALWAYS run:

```bash
cargo build
cargo test
cargo install --path .
```

This ensures the code compiles, tests pass, and the latest binary is available system-wide.

## Project Structure

FireMusic follows the FireSuite `core/` + `cli/` + `tui/` split with a public library crate:

```
firemusic/
├── Cargo.toml              # Binary name: msc, lib: firemusic
├── AGENTS.md
├── README.md
├── CHANGELOG.md
├── scripts/
│   ├── install_linux.sh
│   ├── install_macos.sh
│   └── install_windows.ps1
└── src/
    ├── main.rs             # Thin entry → cli::run()
    ├── lib.rs              # Public API re-exports
    ├── cli/
    │   ├── mod.rs          # Mode routing
    │   ├── args.rs         # clap Args + HelpTopicCmd
    │   ├── play.rs         # Direct playback session
    │   └── help_topics.rs  # msc help <topic> pages
    ├── core/
    │   ├── mod.rs          # Stable public API
    │   ├── error.rs        # Typed errors (thiserror)
    │   ├── config.rs       # Config paths + music dir resolution
    │   ├── paths.rs        # URL/file input validation
    │   ├── mpv.rs          # MpvConfig, create_player, load_inputs
    │   ├── player.rs       # Tactical 3-line UI + play_loop
    │   ├── tactical_select.rs
    │   ├── download.rs
    │   ├── discovery/
    │   │   ├── mod.rs
    │   │   ├── types.rs    # SearchResult, providers, YtdlInfo
    │   │   ├── ytdl.rs     # yt-dlp search
    │   │   ├── tiktok.rs   # TikTok search fallbacks
    │   │   └── session.rs  # Interactive search hub
    │   └── audio/
    │       ├── mod.rs
    │       ├── eq.rs
    │       └── crossfade.rs
    └── tui/
        ├── mod.rs          # run_tui() entry + terminal setup
        ├── app.rs          # AppState, library, playlists, queue
        ├── event_loop.rs   # Event loop + key handlers
        └── ui.rs           # ratatui rendering
```

## Operating Modes

| Mode | Flag | Entry |
|------|------|-------|
| Direct play | (default) | `msc <inputs...>` |
| Discovery | `-s` / `--search` | `handle_search()` |
| Download | `-d` / `--download` | `handle_download()` |
| TUI | `-t` / `--tui` | `tui::run_tui()` |
| Help topics | `msc help <topic>` | `help_topics::*` |

Modes are mutually exclusive at startup. `cli::run()` routes to exactly one path.

## External Dependencies

These are **runtime requirements**, not Rust crates:

| Tool | Purpose |
|------|---------|
| `libmpv` 2.0+ | Audio playback engine |
| `yt-dlp` | Stream resolution, search, download |
| `ffmpeg` | Cover art extraction (TUI, local files only) |

Optional environment:

| Variable | Purpose |
|----------|---------|
| `BRAVE_SEARCH_API_KEY` | Reliable TikTok search via Brave API |

Install on Linux (Debian/Ubuntu):

```bash
sudo apt install libmpv-dev yt-dlp ffmpeg
```

## Configuration Paths

| Data | Path |
|------|------|
| Playlists | `~/.config/firemusic/playlists/*.m3u` |
| EQ presets | `~/.config/firemusic/presets/*.json` |
| Default library | `~/Music` via `core/config.rs` (override with `-m` / `--music-dir`) |

All config paths are resolved through `core/config.rs`. FireMusic does not yet use `/firefly/config/firemusic/` (listed in suite CONTEXT.md as future production path).

## Code Style Guidelines

### Imports & Dependencies

- Order: `std` → external crates → `crate::` modules
- Core internals: `core::error::Result` with `thiserror`; CLI/TUI boundaries: `anyhow::Result`
- Terminal: `crossterm` for raw mode, cursor, events
- TUI: `ratatui` + `CrosstermBackend`
- Playback: `libmpv2::Mpv` — never spawn `mpv` as subprocess for playback
- External tools: `std::process::Command` for `yt-dlp` and `ffmpeg` only

### Types & Naming

- **Structs**: PascalCase (`SearchResult`, `AppState`, `EqState`)
- **Functions**: snake_case (`play_loop`, `handle_search`, `tactical_select`)
- **Enums**: PascalCase (`PlayLoopResult`, `Tab`, `Focus`, `InputAction`)
- **Constants**: UPPER_SNAKE_CASE (`EQ_PRESETS`, `AUDIO_EXTS`, `PROVIDERS`)

### Error Handling

- `core/error.rs`: typed `Error` enum (`InvalidInput`, `MpvInit`, `MpvCommand`, `ConfigPath`, `Io`)
- `core/paths.rs` and `core/config.rs` return `core::error::Result`
- `cli/` and public `handle_*` functions return `anyhow::Result` (auto-converts via `?`)
- External command failures: log stderr, continue or bail depending on context
- Never `unwrap()` in production paths; `.ok()` is acceptable for non-critical MPV property sets

### CLI Design

- Use `clap` derive macros
- Binary name in help: `firemusic`; user-facing alias: `msc`
- Custom help template with colored title; detailed docs in `msc help <topic>`
- Help topics: `discovery`, `download`, `interface`, `controls`
- `-H` is a visible alias for `-h` / `--help`

## UI Systems

FireMusic has **two distinct UI engines**. Do not mix their rendering logic.

### 1. Tactical CLI UI (`core/player.rs`)

The default playback interface. Strict **3-line fixed block**:

1. Status + title + bitrate
2. Progress bar + volume/speed/pitch/EQ
3. Centered keyboard shortcuts

Rules:

- Always use absolute cursor positioning (`cursor::MoveToColumn(0)`, `MoveUp`)
- Clear lines with `ClearType::CurrentLine` before redraw
- Batch writes with `queue!` + `flush`
- Never `println!` inside the playback loop (except initial spacer lines)
- Status glyphs: `▶` playing, `⏸` paused; progress head: `█` in `dark_red`
- Theme: `dark_red` / `white` / `dark_grey` accents (not cyan)

### 2. TUI (`tui/`)

Full-screen ratatui interface launched with `--tui`.

Layout:

- Sidebar (width >= 96): stats, playback metrics, library path
- Title bar: `🔥 Firemusic (<tab>)`
- Main content: Queue / Library / Playlists / Stats (F1–F4)
- Now playing bar: title, progress, time
- Status bar: keybindings or transient messages

Rules:

- Enter alternate screen on start; restore on exit
- Panic hook must call `disable_raw_mode` + `LeaveAlternateScreen`
- Messages auto-dismiss after 3 seconds (`AppState::clear_old_message`)
- Focus model: `Tab` cycles `List` ↔ `NowPlaying` only
- `Esc` is contextual: close help → back from playlist → clear filter → quit
- Cover art loads in background thread via `mpsc`; never block main loop on `ffmpeg`
- Kitty graphics protocol for cover art; guard with `supports_graphics_protocol()`
- Theme: `LightRed` accent, `Rgb(110, 40, 0)` selection background

### 3. Tactical Select (`core/tactical_select.rs`)

Shared interactive menu engine for discovery, download, and search flows.

Rules:

- Enable raw mode; hide cursor; reserve screen lines upfront
- Paginate lists > 15 items (page size 10)
- `Enter` confirms selection; `Space` toggles multi-select without moving cursor
- `←/→` change pages when paginated
- Always restore cursor and clear menu area on exit
- Truncate prompt and items to terminal width

## Module Guidelines

### `core/player.rs`

- `play_loop()` returns `PlayLoopResult`: `Quit`, `SearchAgain`, `EndReached`
- `render_ui()` is called every ~50ms in the loop
- EQ cycle via `e`; manual EQ overlay via `E` → `eq_mode_overlay()`
- Playlist advance on `MEvent::EndFile` when not looping

### `core/discovery/`

- `types.rs`: `SearchResult`, `PROVIDERS`, `YtdlInfo`, URL normalization
- `ytdl.rs`: yt-dlp search for yt/ym/sc providers
- `tiktok.rs`: Brave API → Bing → DuckDuckGo HTML fallback
- `session.rs`: interactive search hub (`handle_search`)
- Prefix syntax: `sc:query`, `tk:query`
- `SearchResult::get_playable_url()` prefers public URLs over API extractor URLs

### `core/download.rs`

- Modes: `interactive` (default), `audio`, `video`
- Interactive wizard: stream type → formats → metadata extras → subtitle check → execute
- Output template: `%(title).200B [%(id)s].%(ext)s`
- Subtitle embedding only when container supports it (m4a, mp4, mkv)
- Probe subtitles only when user selects "check subtitles"

### `core/audio/eq.rs`

- 10 bands: 31 Hz – 16 kHz
- Presets saved as JSON via `core/config.rs::presets_dir()`
- `apply()` builds MPV `af` filter chain from non-zero gains
- Gain range: -12 dB to +12 dB

### `tui/app.rs`

- `LibraryState`: scans `AUDIO_EXTS` recursively by folder
- `PlaylistManager`: M3U read/write via `core/config.rs::playlists_dir()`
- `EQ_PRESETS` / `EQ_PRESET_NAMES`: quick-cycle presets in TUI (separate from manual EQ)

## Key Bindings

### Tactical playback (CLI mode)

| Key | Action |
|-----|--------|
| Space | Pause / resume |
| ←/→, h/l | Seek ±5s |
| { / } | Seek ±1m |
| ↑/↓, k/j | Volume ±5 |
| +/- | Speed ±0.1 |
| , / . | Pitch ±0.05 |
| 1–9 | Jump to 10%–90% |
| 0 | Reset speed + pitch |
| e | Cycle EQ preset |
| E | Manual EQ overlay |
| l | Toggle loop |
| m | Mute |
| s | Back to search (end of track) |
| q / Esc | Quit |

### TUI

| Key | Action |
|-----|--------|
| F1–F4 | Queue / Library / Playlists / Stats |
| Tab | Cycle List ↔ NowPlaying focus |
| ? | Toggle help popup |
| / | Filter library |
| c | Change library directory |
| r | Rescan library |
| n / s | New / save playlist |
| d | Remove from queue or delete playlist |

## Testing

```bash
# All tests
cargo test

# Discovery module only
cargo test -- discovery::
```

Current test coverage:

- `core/discovery/types.rs`: SoundCloud URL normalization, TikTok ID fallback
- `core/discovery/tiktok.rs`: TikTok URL extraction and deduplication
- `core/paths.rs`: URL scheme validation

When adding search or URL logic, add unit tests in the same module under `#[cfg(test)]`.

CLI and TUI flows are not covered by automated tests (require TTY + MPV).

## DO

- Follow the zero-leak 3-line UI contract in tactical playback mode
- Keep tactical_select and TUI terminal cleanup symmetric (raw mode off, cursor shown)
- Use `core/config.rs` for all config paths (not inline `dirs` calls)
- Run `cargo test` before committing
- Run `cargo install --path .` after completing tasks
- Update `CHANGELOG.md` for user-facing changes
- Match existing red/orange visual theme across CLI, TUI, and menus
- Handle terminal resize via width from `terminal::size()`
- Spawn blocking I/O (ffmpeg, yt-dlp metadata) off the main TUI thread

## DO NOT

- Spawn `mpv` as an external process for playback (use `libmpv2` bindings)
- Add `println!` inside the tactical playback render loop
- Block the TUI main loop on `ffmpeg` or `yt-dlp`
- Use cyan as primary accent (legacy; replaced by red/orange in v0.2.5+)
- Hardcode home paths (use `core/config.rs`)
- Use `unwrap()` on user input or external command results
- Test TUI or tactical_select in non-TTY CI without guards
- Break `msc help <topic>` pages or the slim main `--help` output
- Change binary name in `Cargo.toml` (`msc`) without updating install scripts

## Versioning & Changelog

- Version lives in `Cargo.toml` and `cli/args.rs` clap `version` field — keep both in sync
- Document changes in `CHANGELOG.md` following Keep a Changelog format
- ROADMAP.md may lag behind implementation; verify against code before citing it

## Integration with Fire Suite

| Suite app | Integration |
|-----------|-------------|
| **fly** | Install via `fly install firemusic`; preset in fly sources |
| **firesearch** | Planned (`SearchResultType::MusicTrack`) — not yet implemented |
| **firekeep** | Planned (streaming credentials) — not yet implemented |

## Related Documentation

- `README.md` — user-facing usage and examples
- `GEMINI.md` — condensed AI context
- `CHANGELOG.md` — release history
- `../CONTEXT.md` — FireSuite-wide patterns (firemusic now follows core/cli/tui split)
- `../DESIGN.md` — visual identity (TUI follows Fire Red accent where applicable)