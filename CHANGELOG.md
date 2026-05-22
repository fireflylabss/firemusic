# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.5] - 2026-05-22

### Added
- **Inline TUI Input:** Replaced external prompt flow with in-app popup input for library path changes, playlist creation, playlist saving, and library filtering.
- **Library Filtering:** Added `/`-driven filtering in the library view with inline result feedback and `Esc` to clear the filter.
- **TikTok Search Provider:** Added native `tk:` search support in the discovery hub.
- **TikTok Search Fallbacks:** TikTok discovery now tries Brave Search first when `BRAVE_SEARCH_API_KEY` is set, then falls back to public HTML search providers.
- **Search Regression Tests:** Added unit coverage for SoundCloud URL normalization and TikTok URL extraction/fallback behavior.

### Changed
- **Rust Edition:** Ported the crate to Rust 2024 and updated edition-sensitive iterator patterns for compatibility.
- **TUI Focus Model:** Removed the invisible `Player` focus and moved playback controls to the visible `NowPlaying` focus. `Tab` now cycles only between `List` and `NowPlaying`.
- **Escape Behavior:** `Esc` is now contextual in the TUI, closing help, returning from playlist contents, or clearing library filters before exiting.
- **Title Bar:** Updated the header to show only `Firemusic (<current section>)`, without appending transient status text after the section label.
- **Library Path Display:** The library view now abbreviates the user home directory as `~/`.
- **Now Playing Layout:** Simplified the now playing panel to focus on title, progress, and time, with playback metrics left to the sidebar.
- **Responsive Layout:** Adjusted sidebar visibility, panel sizing, and now playing height to behave better in narrower terminals.
- **Visual Theme:** Replaced cyan highlights with red/orange accents across the TUI, tactical selectors, EQ overlay, search results, and download flow.
- **Discovery Hub Documentation:** Updated CLI help and README examples to reflect TikTok search support and the optional `BRAVE_SEARCH_API_KEY`.
- **Download Output Naming:** Downloaded files now use a safer `yt-dlp` output template based on title and media id instead of raw unsanitized titles.
- **Download Extra Features:** Subtitle availability is now checked only when `check subtitles` is selected in the extra features step, instead of probing automatically before the menu.

### Fixed
- **Cover Art Loading:** Moved cover art extraction off the main TUI path to avoid blocking the interface while `ffmpeg` runs.
- **Kitty Cover Placement:** Constrained Kitty cover rendering to the now playing area and only when enough horizontal space is available.
- **Stats Accuracy:** Removed the fake fixed-duration estimate from stats and fall back to `unknown` when real duration data is unavailable.
- **Render Order:** Corrected now playing block rendering so borders and inner content are drawn in a stable order.
- **Initial TUI Paint:** Clears and renders the terminal UI immediately on startup so the shortcut bar does not briefly inherit stale terminal colors before the first keypress.
- **SoundCloud Search URLs:** Search results now prefer public `soundcloud.com/...` pages over internal `api.soundcloud.com/...` extractor URLs when sending media to playback or download.
- **TikTok URL Compatibility:** TikTok fallback URLs now use an extractor-compatible `@_/video/<id>` form for direct `yt-dlp` resolution.
- **TikTok Search Reliability:** Detection of blocked/captcha HTML results now avoids silently returning empty success paths.
- **Metadata Parsing Robustness:** Download metadata parsing now tolerates missing/null `formats` arrays and prints extractor stderr when metadata resolution fails.

## [0.2.4] - 2026-04-04

### Fixed
- **Installation URLs:** Renamed `install_unix.sh` to `install_linux.sh` and created `install_macos.sh` to fix 404 errors during remote installation.
- **Dependency Handling:** Installer scripts now operate in "Strict Diagnostic Mode"—detecting missing requirements (Rust, MPV, yt-dlp) and providing installation instructions instead of forcing automatic installs.
- **Isolated Build Environment:** Installation builds now occur in a controlled temporary directory (`~/.fireflylabs/.tmp`) to ensure `Cargo.toml` is found regardless of the execution path.

## [0.2.3] - 2026-04-04

### Added
- **Full Windows Support:** Dedicated tactical installer (`scripts/install_windows.ps1`).
- **Automated Windows Dependencies:** Installer now automatically handles `yt-dlp.exe` and `libmpv` (DLLs/Libs) downloads and extraction.
- **Unified Unix Installer:** Merged Linux and macOS scripts into a single, smarter `scripts/install_unix.sh`.
- **Uninstallation Guides:** Added comprehensive removal instructions for all platforms in all README files.
- **One-Liner Commands:** Enabled "copy-paste" installation directly from GitHub for all OS.

### Fixed
- **Build Optimization:** Simplified `Cargo.toml` binary targets to prevent redundant compilation and fix PowerShell redirection errors.
- **Windows Linker:** Fixed `LNK1181` error by explicitly exposing library paths to the MSVC linker during installation.
- **Path Handling:** Switched to absolute pathing in scripts to ensure reliability across different terminal environments.
- **Documentation:** Cleaned up redundant text and formatting issues in multilingual README files.

## [0.2.2] - 2026-04-03

### Added
- Installation scripts for Linux and macOS (`scripts/install_linux.sh` and `scripts/install_macos.sh`).
- Automated dependency check (Rust, MPV, yt-dlp) in installation scripts.
- Binary isolation in `~/.fireflylabs/firemusic/` with system path linking.

## [0.2.1] - 2026-04-03

### Changed
- **Help Interface:** Re-organized the `--help` output into a more detailed, yet clean, spaced layout.
- **License:** Migrated from GNU GPL v3 to **Apache License 2.0** (Copyright © 2026-Present Firefly Labs).
- **Documentation:** Updated all multilingual READMEs with `<kbd>` styling and license details.

## [0.2.0] - 2026-04-03

### Added
- **Multi-Format Downloads:** Users can now select multiple audio/video formats at once in the interactive wizard.
- **Granular Stream Control:** Options for `audio only`, `video only`, or `both` streams.
- **Independent Metadata:** Custom filenames can be set individually for audio and video streams when downloading both.
- **Tactical UI Integration:** Control keys are now integrated directly into the progress line (e.g., `volume [↑/↓]`).
- **Shortcuts Guide:** Live UI now uses actual arrow symbols (`←`, `→`, `↑`, `↓`) and tactical brackets `[]` for all key references.

### Changed
- **UI Logic:** The playback status (e.g., `[playing | mute]`) is now at the start of the "now playing" line.
- **Playlist Handling:** Improved sequential playback logic; the app now automatically exits after the last track in the playlist.

## [0.1.7] - 2026-04-03

### Added
- Experimental `yt-dlp` integration for downloading audio/video.
- Interactive download wizard with `--download`.
- Preset download modes: `--download=audio` and `--download=video`.
- Format detection and metadata options in the interactive wizard.
- `dialoguer` dependency for interactive CLI prompts.
- `serde_json` and `serde` dependencies for metadata parsing.

## [0.1.6] - 2026-04-03

### Changed
- UI: Moved playback status indicator *before* the title (e.g., `[playing] title`).
- UI: Mute status integrated into the status tag (e.g., `[playing | mute]`).
- Logic: Automatically close the application when the playlist finishes (sequential playback).
- Features: Support for multiple files/URLs (playlist support).

### Removed
- BPM indicator from the UI.

## [0.1.5] - 2026-04-03

### Added
- Tactical UI styling: Shortcuts now enclosed in `[]`.
- Arrow symbols `←`, `→`, `↑`, `↓` in the UI guide.
- Integrated status line: `playing`, `mute`, `loop` and `bitrate` in one line.

## [0.1.4] - 2026-04-03

### Changed
- UI: Added clear separators (`|`) and increased spacing for better readability.
- UI: Labels like `volume:` and `speed:` are now full words.

## [0.1.3] - 2026-04-03

### Added
- Percentage seeking: `1-9` keys jump to 10%-90%.
- Minute seeking: `[` and `]` keys for 1m jumps.
- Vim keys: `h/l` for seek and `j/k` for volume.
- Dynamic loop toggle: `l` key.
- Reset speed: `0` key.
- Handled `Ctrl+C` to force-kill the process and restore terminal state.

## [0.1.2] - 2026-04-03

### Fixed
- Deep architectural overhaul of the UI engine to prevent "leaking" lines.
- Switched to `queue!` batching and absolute cursor positioning for a "Zero-Leak" 3-line UI.
- Atomic UI updates to handle terminal resizing better.

### Added
- Native web stream support via MPV `ytdl` integration.
- Metadata extraction: UI displays `media-title` for streams.
- Re-designed `--help` with a minimalist, lowercase template.

## [0.1.1] - 2026-04-03

### Added
- Monochromatic, high-density UI layout (White/Dark-Grey).
- Technical metadata: Bitrate (kbps) and Sample Rate (kHz) display.
- Playback speed controls: `+` and `-` keys.
- CLI arguments: `--speed` and `--volume`.

## [0.1.0] - 2021-04-03

### Added
- Initial release of **FireMusic**.
- Simple CLI player using `libmpv`.
- Basic playback controls: space (pause), arrows (seek), m (mute), q (quit).
- Live progress bar in terminal.
