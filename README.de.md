[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 Fire Music

**Fire Music** ist ein leistungsstarker, minimalistischer CLI-Musikplayer, der mit Rust entwickelt wurde und auf der **MPV**-Engine basiert. Er verfügt über eine taktische "Zero-Leak"-Terminaloberfläche, die für Profis entwickelt wurde, die Informationen mit hoher Dichte ohne Terminal-Chaos wünschen.

---

## 🚀 Hauptmerkmale

- **Zero-Leak-UI-Engine:** Ein streng verwalteter, fester 3-Zeilen-UI-Block. Er verwendet absolute Cursor-Positionierung und atomare Updates, um sicherzustellen, dass Ihr Terminal-Verlauf zu 100 % sauber bleibt.
- **Erweiterter Download-Assistent:** Integriertes `yt-dlp`-System. Verwenden Sie `--download`, um einen interaktiven Assistenten zu starten, der Folgendes unterstützt:
    - Granulare Stream-Auswahl (Audio, Video oder Beides).
    - Multi-Format-Warteschlange (wählen Sie mehrere Erweiterungen wie `mp3` + `flac` gleichzeitig aus).
    - Unabhängige Dateinamen-Vorlagen für verschiedene Streams.
    - Automatische Auflösungserkennung.
- **Web-Stream-Wiedergabe:** Native Integration mit `yt-dlp`, die eine direkte Wiedergabe von YouTube, SoundCloud und Tausenden anderen Anbietern ermöglicht.
- **Sequenzielle Playlists:** Geben Sie mehrere Dateien oder URLs an. Der Player spielt diese nacheinander ab und beendet sich nach dem letzten Titel automatisch.
- **Taktisches Feedback:** Bitraten-Tracking in Echtzeit und Live-Extraktion von Medientiteln.
- **Vim-freundlich:** Volle Unterstützung der Bewegungstasten `h/j/k/l`.

---

## 🛠 Voraussetzungen

Stellen Sie vor der Installation sicher, dass die `libmpv`-Entwicklungsdateien (Version 2.0 oder kompatibel) und `yt-dlp` auf Ihrem System installiert sind.

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

### Automatisiertes Skript (Empfohlen)
Sie können **Fire Music** zusammen mit allen Abhängigkeiten (einschließlich Rust und MPV) mit den folgenden Befehlen installieren:

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_unix.sh | bash
```

**Windows:**
```powershell
powershell -c "irm https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_windows.ps1 | iex"
```

### Manuelle Installation
Um den Player zu installieren und die System-Aliase (`firemusic`, `msc`, `frmsc`) zu registrieren, führen Sie folgenden Befehl im Projektverzeichnis aus:

```bash
cargo install --path .
```

---

## ⌨️ Benutzung

Sie können jeden der bereitgestellten Aliase verwenden: `msc`, `firemusic` oder `frmsc`.

```bash
msc <Datei-oder-URL> [FLAGS]
```

### Argumente & Flags
| Flag | Kurz | Beschreibung | Standard |
| :--- | :--- | :--- | :--- |
| `--loop` | <kbd>-l</kbd> | Den gesamten Titel/die Playlist unendlich wiederholen | `false` |
| `--speed` | <kbd>-s</kbd> | Anfänglicher Wiedergabegeschwindigkeitsfaktor (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Anfänglicher Lautstärkepegel in Prozent (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Startet den interaktiven Multi-Format-Download-Assistenten | - |

---

## 🎹 Taktische Tastenkürzel

Während der Wiedergabe bietet die Benutzeroberfläche taktisches Echtzeit-Feedback. Verwenden Sie die folgenden Tasten:

| Taste | Aktion |
| :--- | :--- |
| <kbd>Leertaste</kbd> / <kbd>p</kbd> | **Pause / Fortsetzen** |
| <kbd>←</kbd> / <kbd>→</kbd> | **5s Suchen** rückwärts / vorwärts (auch <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>[</kbd> / <kbd>]</kbd> | **1m Suchen** rückwärts / vorwärts |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Lautstärke** erhöhen / verringern (auch <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Springen** zu Prozentsatz (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Geschwindigkeit** erhöhen / verringern |
| <kbd>0</kbd> | **Zurücksetzen** der Geschwindigkeit auf 1.0x |
| <kbd>l</kbd> | **Loop-Modus** während des Betriebs umschalten |
| <kbd>m</kbd> | **Stummschaltung** umschalten |
| <kbd>q</kbd> / <kbd>Strg+c</kbd> | **Beenden** der Sitzung |

---

## 🎥 Download-Beispiele

### Interaktiver Assistent
Führen Sie einfach den Befehl aus und folgen Sie den taktischen Eingabeaufforderungen, um Streams, Formate und Metadaten-Optionen auszuwählen:
```bash
msc --download
```

### Hochgeschwindigkeits-Presets
Laden Sie hochwertige Versionen direkt ohne den Assistenten herunter:
```bash
# Hochwertiges MP3 (320kbps)
msc --download=audio "https://youtube.com/watch?v=..."

# Beste Videoqualität (MP4)
msc --download=video "https://youtube.com/watch?v=..."
```

---

## 🗑️ Deinstallation

### Windows
- Löschen Sie den Ordner: `C:\Users\<Benutzer>\.fireflylabs\firemusic`
- (Optional) Entfernen Sie den Ordner aus Ihrem Benutzer-PATH in den Umgebungsvariablen.

### Linux / macOS
- Löschen Sie die Binärdatei: `rm $(which msc)`
- (Falls über Cargo installiert): `cargo uninstall firemusic`

---

## 📜 Lizenz

Dieses Projekt steht unter der **Apache License 2.0**. Weitere Details finden Sie in der Datei `LICENSE`.

Copyright © 2026-Present Firefly Labs
