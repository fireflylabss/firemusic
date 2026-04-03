[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 Fire Music

**Fire Music** is a high-performance, minimalist CLI music player built with Rust and powered by the **MPV** engine. It features a tactical, "zero-leak" terminal interface designed for pro users who want high-density information without terminal clutter.

---

## 🚀 Key Features

- **Zero-Leak UI Engine:** A strictly managed 3-line fixed UI block. It uses absolute cursor positioning and atomic updates to ensure your terminal scrollback remains 100% clean.
- **Advanced Download Wizard:** Integrated `yt-dlp` system. Use `--download` to start an interactive wizard that supports:
    - Granular stream selection (Audio, Video, or Both).
    - Multi-format queuing (select multiple extensions like `mp3` + `flac` at once).
    - Independent filename templates for different streams.
    - Automatic resolution detection.
- **Web Stream Playback:** Native integration with `yt-dlp` allowing direct playback from YouTube, SoundCloud, and thousands of other providers.
- **Sequential Playlists:** Pass multiple files or URLs. The player will advance through them and automatically exit after the final track.
- **Tactical Feedback:** Real-time bitrate tracking and live media title extraction.
- **Vim-Friendly:** Full support for `h/j/k/l` movement keys.

---

## 🛠 Prerequisites

Before installing, ensure you have the `libmpv` development files (version 2.0 or compatible) and `yt-dlp` installed on your system.

### Linux (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install libmpv-dev yt-dlp
```

### macOS
```bash
brew install mpv yt-dlp
```

---

## 📦 Installation

To install the player and register the system aliases (`firemusic`, `msc`, `frmsc`), run the following in the project root:

```bash
cargo install --path .
```

---

## ⌨️ Usage

You can use any of the provided aliases: `msc`, `firemusic`, or `frmsc`.

```bash
msc <file-or-url> [FLAGS]
```

### Arguments & Flags
| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--loop` | <kbd>-l</kbd> | Repeat the entire track/playlist infinitely | `false` |
| `--speed` | <kbd>-s</kbd> | Initial playback speed factor (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Initial volume level percentage (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Start the interactive multi-format download wizard | - |

---

## 🎹 Tactical Shortcuts

During playback, the UI provides real-time tactical feedback. Use the following keys:

| Key | Action |
| :--- | :--- |
| <kbd>space</kbd> / <kbd>p</kbd> | **Pause / Resume** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Seek 5s** backward / forward (also <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>[</kbd> / <kbd>]</kbd> | **Seek 1m** backward / forward |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volume** up / down (also <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Jump** to percentage (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Speed** increase / decrease |
| <kbd>0</kbd> | **Reset** playback speed to 1.0x |
| <kbd>l</kbd> | **Toggle Loop** mode on the fly |
| <kbd>m</kbd> | **Toggle Mute** |
| <kbd>q</kbd> / <kbd>ctrl+c</kbd> | **Quit** session |

---

## 🎥 Download Examples

### Interactive Wizard
Simply run the command and follow the tactical prompts to select streams, formats, and metadata options:
```bash
msc --download
```

### High-Speed Presets
Download high-quality versions directly without the wizard:
```bash
# High-quality MP3 (320kbps)
msc --download=audio "https://youtube.com/watch?v=..."

# Best quality Video (MP4)
msc --download=video "https://youtube.com/watch?v=..."
```

---

## 📜 License

This project is licensed under the **Apache License 2.0**. See the `LICENSE` file for details.

Copyright © 2026-Present Firefly Labs
