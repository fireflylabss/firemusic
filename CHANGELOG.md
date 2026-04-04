# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.1.7] - 2026-04-03

### Added
- Experimental `yt-dlp` integration for downloading audio/video.
- Interactive download wizard with `--download`.
- Preset download modes: `--download=audio` and `--download=video`.
- Format detection and metadata options in the interactive wizard.
- `dialoguer` dependency for interactive CLI prompts.
- `serde_json` and `serde` dependencies for metadata parsing.
