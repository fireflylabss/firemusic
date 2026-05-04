[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 FireMusic

**FireMusic** é um player de música CLI de alto desempenho e minimalista, construído com Rust e alimentado pelo motor **MPV**. Ele apresenta uma interface de terminal tática "zero-leak", projetada para usuários avançados que desejam informações de alta densidade sem poluir o terminal.

---

## 🚀 Principais Características

- **Motor de UI Zero-Leak:** Um bloco de UI fixo de 3 linhas estritamente gerenciado. Utiliza posicionamento absoluto do cursor e atualizações atômicas para garantir que o histórico do seu terminal permaneça 100% limpo.
- **Discovery Hub Interativo:** Pesquise músicas no **YouTube**, **YouTube Music** e **SoundCloud** diretamente do CLI. Selecione e toque ou baixe resultados em lote.
- **Audio FX & Pitch:** Manipulação em tempo real do **Pitch** (frequência) e presets de **Equalizador** (Bass+, Treble+, Rock, Vocal, Lofi).
- **Assistente de Download Avançado:** Sistema `yt-dlp` integrado. Use `--download` para iniciar um assistente interativo que suporta:
    - Seleção granular de stream (Áudio, Vídeo ou Ambos).
    - Fila multiformato (selecione múltiplas extensões como `mp3` + `flac` de uma vez).
    - Incorporação de metadados, capas e legendas.
    - Validação inteligente de legendas baseada no formato.
- **Reprodução de Streams Web:** Integração nativa com `yt-dlp`, permitindo a reprodução direta de milhares de provedores.
- **Navegação Inteligente:** Paginação interativa para grandes resultados de busca e menu de decisão pós-faixa.

---

## 🛠 Pré-requisitos

Antes de instalar, certifique-se de ter os arquivos de desenvolvimento da `libmpv` (versão 2.0 ou compatível) e o `yt-dlp` instalados em seu sistema.

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

## 📦 Instalação

### Instalação Manual
Para instalar o player e registrar os aliases do sistema (`firemusic`, `msc`), execute o seguinte na raiz do projeto:

```bash
cargo install --path .
```

---

## ⌨️ Uso

Você pode usar tanto `msc` quanto `firemusic`.

```bash
msc <arquivo-ou-url> [FLAGS]
```

### Argumentos & Flags
| Flag | Curta | Descrição | Padrão |
| :--- | :--- | :--- | :--- |
| `--search` | <kbd>-s</kbd> | Abrir o Discovery Hub Interativo | - |
| `--loop` | <kbd>-l</kbd> | Repetir a faixa/playlist inteira infinitamente | `false` |
| `--speed` | <kbd>-f</kbd> | Fator de velocidade inicial (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Porcentagem do nível de volume inicial (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Iniciar o assistente de download interativo | - |

---

## 🎹 Atalhos Táticos

Durante a reprodução, a UI fornece feedback tático em tempo real. Use as seguintes teclas:

| Tecla | Ação |
| :--- | :--- |
| <kbd>space</kbd> | **Pausar / Retomar** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Pular 5s** para trás / frente (também <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>{</kbd> / <kbd>}</kbd> | **Pular 1m** para trás / frente |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volume** aumentar / diminuir (também <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Pular** para porcentagem (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Velocidade** aumentar / diminuir |
| <kbd>,</kbd> / <kbd>.</kbd> | **Pitch** diminuir / aumentar |
| <kbd>e</kbd> | **Ciclar Equalizador** (Bass+, Treble+, Rock, Vocal, Lofi, Off) |
| <kbd>0</kbd> | **Resetar** todos os FX (velocidade, pitch, eq) para o padrão |
| <kbd>l</kbd> | **Alternar Loop** em tempo real |
| <kbd>m</kbd> | **Alternar Mudo** |
| <kbd>s</kbd> | **Voltar para Busca** (apenas no fim da faixa) |
| <kbd>q</kbd> / <kbd>esc</kbd> | **Sair** da sessão |

---

## 🔍 Exemplos do Discovery Hub

Pesquise e toque música sem sair do terminal:
```bash
# Abrir hub e escolher provedor
msc -s

# Busca rápida no YouTube
msc -s "daft punk"

# Busca rápida no SoundCloud
msc -s "sc:lofi beats"
```

---

## 📦 Exemplos de Download

### Assistente Interativo
```bash
msc --download
```

### Presets de Alta Velocidade
```bash
# MP3 de alta qualidade
msc --download=audio "URL"

# MP4 em 1080p
msc --download=video "URL"
```

---

## 📜 Licença

Este projeto está sob a Licença **Apache 2.0**.

Copyright © 2026-Present Firefly Labs
