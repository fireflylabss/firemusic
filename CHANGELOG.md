# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-04-03

### Changed
- **Help Interface:** Re-organized the `--help` output into a more detailed, yet clean, spaced layout.
- **License:** Migrated from GNU GPL v3 to **Apache License 2.0** (Copyright © 2026-Present Firefly Labs).
- **Documentation:** Updated all multilingual READMEs with `<kbd>` styling and license details.

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

### Fixed
- UI ownership issues and unreachable patterns in keyboard handling.

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
- Initial release of **Fire Music**.
- Simple CLI player using `libmpv`.
- Aliases: `firemusic`, `msc`, `frmsc`.
- Basic playback controls: space (pause), arrows (seek), m (mute), q (quit).
- Live progress bar in terminal.
