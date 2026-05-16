# 🔥 FireMusic

**FireMusic** is a high-performance, minimalist CLI music player built with Rust and powered by the **MPV** engine. It features a tactical, "zero-leak" terminal interface designed for pro users who want high-density information without terminal clutter.

---

## 🚀 Key Features

- **Zero-Leak UI Engine:** A strictly managed 3-line fixed UI block. It uses absolute cursor positioning and atomic updates to ensure your terminal scrollback remains 100% clean.
- **Interactive Discovery Hub:** Search music across **YouTube**, **YouTube Music**, and **SoundCloud** directly from the CLI. Select and play or batch-download results.
- **Audio FX & Pitch:** Real-time manipulation of playback **Pitch** (frecuency) and **Equalizer** presets (Bass+, Treble+, Rock, Vocal, Lofi).
- **Advanced Download Wizard:** Integrated `yt-dlp` system. Use `--download` to start an interactive wizard that supports:
    - Granular stream selection (Audio, Video or Both).
    - Multi-format queuing (select multiple extensions like `mp3` + `flac` at once).
    - Metadata, thumbnail, and subtitle embedding.
    - Format-aware validation for embedded subtitles.
- **Web Stream Playback:** Native integration with `yt-dlp` allowing direct playback from thousands of providers.
- **Smart Navigation:** Interactive pagination for large search results and a post-track decision menu.

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

### Manual Installation
To install the player and register the system alias (`firemusic`), run the following in the project root:

```bash
cargo install --path .
```

---

## ⌨️ Usage

You can use either `msc` or `firemusic`.

```bash
msc <file-or-url> [FLAGS]
```

### Arguments & Flags
| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--search` | <kbd>-s</kbd> | Open the Interactive Discovery Hub | - |
| `--loop` | <kbd>-l</kbd> | Repeat the entire track/playlist infinitely | `false` |
| `--speed` | <kbd>-f</kbd> | Initial playback speed factor (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Initial volume level percentage (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Start the interactive multi-format download wizard | - |

---

## 🎹 Tactical Shortcuts

During playback, the UI provides real-time tactical feedback. Use the following keys:

| Key | Action |
| :--- | :--- |
| <kbd>space</kbd> | **Pause / Resume** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Seek 5s** backward / forward (also <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>{</kbd> / <kbd>}</kbd> | **Seek 1m** backward / forward |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volume** up / down (also <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Jump** to percentage (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Speed** increase / decrease |
| <kbd>,</kbd> / <kbd>.</kbd> | **Pitch** decrease / increase |
| <kbd>e</kbd> | **Cycle Equalizer** (Bass+, Treble+, Rock, Vocal, Lofi, Off) |
| <kbd>0</kbd> | **Reset** all FX (speed, pitch, eq) to defaults |
| <kbd>l</kbd> | **Toggle Loop** mode on the fly |
| <kbd>m</kbd> | **Toggle Mute** |
| <kbd>s</kbd> | **Back to Search** (only at the end of track) |
| <kbd>q</kbd> / <kbd>esc</kbd> | **Quit** session |

---

## 🔍 Discovery Hub Examples

Search and play music without leaving the terminal:
```bash
# Open hub and choose provider
msc -s

# Quick search on YouTube
msc -s "daft punk"

# Quick search on SoundCloud
msc -s "sc:lofi beats"
```

---

## 📦 Download Examples

### Interactive Wizard
```bash
msc --download
```

### High-Speed Presets
```bash
# High-quality MP3
msc --download=audio "URL"

# 1080p MP4
msc --download=video "URL"
```

---

## 📜 License

This project is licensed under the **Apache License 2.0**.

Copyright © 2026-Present Firefly Labs
