[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 FireMusic

**FireMusic** es un reproductor de música CLI de alto rendimiento y minimalista, construido con Rust y potenciado por el motor **MPV**. Cuenta con una interfaz de terminal táctica "zero-leak" diseñada para usuarios avanzados que buscan información de alta densidad sin desordenar el terminal.

---

## 🚀 Características Principales

- **Motor de UI Zero-Leak:** Un bloque de UI fijo de 3 líneas estrictamente gestionado. Utiliza posicionamiento absoluto del cursor y actualizaciones atómicas para asegurar que el historial de su terminal permanezca 100% limpio.
- **Asistente de Descarga Avanzado:** Sistema `yt-dlp` integrado. Use `--download` para iniciar un asistente interactivo que admite:
    - Selección granular de flujo (Audio, Video o Ambos).
    - Cola multiformato (seleccione múltiples extensiones como `mp3` + `flac` a la vez).
    - Plantillas de nombre de archivo independientes para diferentes flujos.
    - Detección automática de resolución.
- **Reproducción de Streams Web:** Integración nativa con `yt-dlp` que permite la reproducción directa desde YouTube, SoundCloud y miles de otros proveedores.
- **Listas de Reproducción Secuenciales:** Pase múltiples archivos o URLs. El reproductor avanzará a través de ellos y se cerrará automáticamente después de la pista final.
- **Feedback Táctico:** Seguimiento de bitrate en tiempo real y extracción de títulos multimedia en vivo.
- **Compatible con Vim:** Soporte completo para las teclas de movimiento `h/j/k/l`.

---

## 🛠 Prerrequisitos

Antes de instalar, asegúrese de tener los archivos de desarrollo de `libmpv` (versión 2.0 o compatible) y `yt-dlp` instalados en su sistema.

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

## 📦 Instalación

### Script Automatizado (Recomendado)
Puede instalar **FireMusic** junto com todas sus dependencias (incluyendo Rust y MPV) usando los siguientes comandos:

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_unix.sh | bash
```

**Windows:**
```powershell
powershell -c "irm https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_windows.ps1 | iex"
```

### Instalación Manual
Para instalar el reproductor y registrar los alias del sistema (`firemusic`, `msc`, `frmsc`), ejecute lo siguiente en la raíz del proyecto:

```bash
cargo install --path .
```

---

## ⌨️ Uso

Puede usar cualquiera de los alias proporcionados: `msc`, `firemusic`, o `frmsc`.

```bash
msc <archivo-o-url> [FLAGS]
```

### Argumentos y Banderas
| Bandera | Corta | Descripción | Predeterminado |
| :--- | :--- | :--- | :--- |
| `--loop` | <kbd>-l</kbd> | Repetir la pista/lista de reproducción infinitamente | `false` |
| `--speed` | <kbd>-s</kbd> | Factor de velocidad de reproducción inicial (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Porcentaje de nivel de volumen inicial (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Iniciar el asistente de descarga interactivo multiformato | - |

---

## 🎹 Atajos Tácticos

Durante la reproducción, la interfaz proporciona feedback táctico en tiempo real. Use las siguientes teclas:

| Tecla | Acción |
| :--- | :--- |
| <kbd>space</kbd> / <kbd>p</kbd> | **Pausa / Reanudar** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Búsqueda 5s** atrás / adelante (también <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>[</kbd> / <kbd>]</kbd> | **Búsqueda 1m** atrás / adelante |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volumen** subir / bajar (también <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Saltar** al porcentaje (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Velocidad** aumentar / disminuir |
| <kbd>0</kbd> | **Reiniciar** velocidad de reproducción a 1.0x |
| <kbd>l</kbd> | **Alternar Bucle** sobre la marcha |
| <kbd>m</kbd> | **Alternar Silencio** |
| <kbd>q</kbd> / <kbd>ctrl+c</kbd> | **Salir** de la sesión |

---

## 🎥 Ejemplos de Descarga

### Asistente Interactivo
Simplemente ejecute el comando y siga las instrucciones tácticas para seleccionar flujos, formatos y opciones de metadatos:
```bash
msc --download
```

### Ajustes Preestablecidos de Alta Velocidad
Descargue versiones de alta calidad directamente sin el asistente:
```bash
# MP3 de alta calidad (320kbps)
msc --download=audio "https://youtube.com/watch?v=..."

# Video de mejor calidad (MP4)
msc --download=video "https://youtube.com/watch?v=..."
```

---

## 🗑️ Desinstalación

### Windows
- Elimine la carpeta: `C:\Users\<Usuario>\.fireflylabs\firemusic`
- (Opcional) Elimine la carpeta de su PATH de usuario en las Variables de Entorno.

### Linux / macOS
- Elimine el binario: `rm $(which msc)`
- (Si se instaló a través de cargo): `cargo uninstall firemusic`

---

## 📜 Licencia

Este proyecto está bajo la Licencia **Apache 2.0**. Consulte el archivo `LICENSE` para más detalles.

Copyright © 2026-Present Firefly Labs
