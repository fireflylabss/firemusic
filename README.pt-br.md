[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 Fire Music

**Fire Music** é um player de música CLI de alto desempenho e minimalista, construído com Rust e alimentado pelo motor **MPV**. Ele apresenta uma interface de terminal tática "zero-leak", projetada para usuários avançados que desejam informações de alta densidade sem poluir o terminal.

---

## 🚀 Principais Características

- **Motor de UI Zero-Leak:** Um bloco de UI fixo de 3 linhas estritamente gerenciado. Utiliza posicionamento absoluto do cursor e atualizações atômicas para garantir que o histórico do seu terminal permaneça 100% limpo.
- **Assistente de Download Avançado:** Sistema `yt-dlp` integrado. Use `--download` para iniciar um assistente interativo que suporta:
    - Seleção granular de stream (Áudio, Vídeo ou Ambos).
    - Fila multiformato (selecione múltiplas extensões como `mp3` + `flac` de uma vez).
    - Templates de nome de arquivo independentes para diferentes streams.
    - Detecção automática de resolução.
- **Reprodução de Streams Web:** Integração nativa com `yt-dlp`, permitindo a reprodução direta do YouTube, SoundCloud e milhares de outros provedores.
- **Playlists Sequenciais:** Passe múltiplos arquivos ou URLs. O player avançará por eles e sairá automaticamente após a última faixa.
- **Feedback Tático:** Rastreamento de bitrate em tempo real e extração de títulos de mídia ao vivo.
- **Amigável ao Vim:** Suporte total para as teclas de movimento `h/j/k/l`.

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

### Script Automatizado (Recomendado)
Você pode instalar o **Fire Music** junto com todas as suas dependências (incluindo Rust e MPV) usando os seguintes comandos:

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_unix.sh | bash
```

**Windows:**
```powershell
powershell -c "irm https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_windows.ps1 | iex"
```

### Instalação Manual
Para instalar o player e registrar os aliases do sistema (`firemusic`, `msc`, `frmsc`), execute o seguinte na raiz do projeto:

```bash
cargo install --path .
```

---

## ⌨️ Uso

Você pode usar qualquer um dos aliases fornecidos: `msc`, `firemusic` ou `frmsc`.

```bash
msc <arquivo-ou-url> [FLAGS]
```

### Argumentos & Flags
| Flag | Curta | Descrição | Padrão |
| :--- | :--- | :--- | :--- |
| `--loop` | <kbd>-l</kbd> | Repetir a faixa/playlist inteira infinitamente | `false` |
| `--speed` | <kbd>-s</kbd> | Fator de velocidade de reprodução inicial (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Porcentagem do nível de volume inicial (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Iniciar o assistente de download interativo multiformato | - |

---

## 🎹 Atalhos Táticos

Durante a reprodução, a UI fornece feedback tático em tempo real. Use as seguintes teclas:

| Tecla | Ação |
| :--- | :--- |
| <kbd>space</kbd> / <kbd>p</kbd> | **Pausar / Retomar** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Pular 5s** para trás / frente (também <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>[</kbd> / <kbd>]</kbd> | **Pular 1m** para trás / frente |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volume** aumentar / diminuir (também <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Pular** para porcentagem (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Velocidade** aumentar / diminuir |
| <kbd>0</kbd> | **Resetar** velocidade de reprodução para 1.0x |
| <kbd>l</kbd> | **Alternar Loop** em tempo real |
| <kbd>m</kbd> | **Alternar Mudo** |
| <kbd>q</kbd> / <kbd>ctrl+c</kbd> | **Sair** da sessão |

---

## 🎥 Exemplos de Download

### Assistente Interativo
Basta executar o comando e seguir os prompts táticos para selecionar streams, formatos e opções de metadados:
```bash
msc --download
```

### Presets de Alta Velocidade
Baixe versões de alta qualidade diretamente sem o assistente:
```bash
# MP3 de alta qualidade (320kbps)
msc --download=audio "https://youtube.com/watch?v=..."

# Vídeo de melhor qualidade (MP4)
msc --download=video "https://youtube.com/watch?v=..."
```

---

## 🗑️ Desinstalação

### Windows
- Exclua a pasta: `C:\Users\<Usuario>\.fireflylabs\firemusic`
- (Opcional) Remova a pasta do seu PATH de usuário nas Variáveis de Ambiente.

### Linux / macOS
- Exclua o binário: `rm $(which msc)`
- (Se instalado via cargo): `cargo uninstall firemusic`

---

## 📜 Licença

Este projeto está sob a Licença **Apache 2.0**. Veja o arquivo `LICENSE` para mais detalhes.

Copyright © 2026-Present Firefly Labs
