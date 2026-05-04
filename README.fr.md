[English](README.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-br.md) | [Deutsch](README.de.md) | [Français](README.fr.md)

# 🔥 FireMusic

**FireMusic** est un lecteur de musique CLI ultra-performant et minimaliste, écrit en Rust et propulsé par le moteur **MPV**. Il dispose d'une interface de terminal tactique "zero-leak" conçue pour les utilisateurs avancés qui souhaitent des informations haute densité sans encombrer le terminal.

---

## 🚀 Caractéristiques Principales

- **Moteur UI Zero-Leak :** Un bloc UI fixe de 3 lignes strictement géré. Il utilise un positionnement absolu du curseur et des mises à jour atomiques pour garantir que l'historique de votre terminal reste 100% propre.
- **Assistant de Téléchargement Avancé :** Système `yt-dlp` intégré. Utilisez `--download` pour lancer un assistant interactif qui prend en charge :
    - Sélection granulaire des flux (Audio, Vidéo ou les deux).
    - File d'attente multi-format (sélectionnez plusieurs extensions comme `mp3` + `flac` à la fois).
    - Modèles de noms de fichiers indépendants pour les différents flux.
    - Détection automatique de la résolution.
- **Lecture de Flux Web :** Intégration native avec `yt-dlp` permettant la lecture directe depuis YouTube, SoundCloud et des milliers d'autres plateformes.
- **Listes de Lecture Séquentielles :** Passez plusieurs fichiers ou URLs. Le lecteur les parcourra et s'arrêtera automatiquement après la dernière piste.
- **Feedback Tactique :** Suivi du débit binaire (bitrate) en temps réel et extraction des titres média en direct.
- **Vim-Friendly :** Support complet des touches de mouvement `h/j/k/l`.

---

## 🛠 Prérequis

Avant l'installation, assurez-vous d'avoir les fichiers de développement de `libmpv` (version 2.0 ou compatible) et `yt-dlp` installés sur votre système.

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

### Script Automatisé (Recommandé)
Vous pouvez installer **FireMusic** ainsi que toutes ses dépendances (y compris Rust et MPV) à l'aide des commandes suivantes :

**Linux / macOS :**
```bash
curl -sSL https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_unix.sh | bash
```

**Windows :**
```powershell
powershell -c "irm https://raw.githubusercontent.com/fireflylabss/firemusic/main/scripts/install_windows.ps1 | iex"
```

### Installation Manuelle
Pour installer le lecteur et enregistrer les alias système (`firemusic`, `msc`, `frmsc`), exécutez la commande suivante à la racine du projet :

```bash
cargo install --path .
```

---

## ⌨️ Utilisation

Vous pouvez utiliser n'importe quel alias fourni : `msc`, `firemusic`, ou `frmsc`.

```bash
msc <fichier-ou-url> [FLAGS]
```

### Arguments & Options
| Option | Court | Description | Défaut |
| :--- | :--- | :--- | :--- |
| `--loop` | <kbd>-l</kbd> | Répéter la piste/liste de lecture à l'infini | `false` |
| `--speed` | <kbd>-s</kbd> | Facteur de vitesse de lecture initial (0.01 - 100.0) | `1.0` |
| `--volume` | <kbd>-v</kbd> | Pourcentage du niveau de volume initial (0 - 100) | `100.0` |
| `--download`| <kbd>-d</kbd> | Lancer l'assistant de téléchargement interactif multi-format | - |

---

## 🎹 Raccourcis Tactiques

Pendant la lecture, l'interface fournit un feedback tactique en temps réel. Utilisez les touches suivantes :

| Touche | Action |
| :--- | :--- |
| <kbd>espace</kbd> / <kbd>p</kbd> | **Pause / Reprendre** |
| <kbd>←</kbd> / <kbd>→</kbd> | **Recherche 5s** arrière / avant (aussi <kbd>h</kbd> / <kbd>l</kbd>) |
| <kbd>[</kbd> / <kbd>]</kbd> | **Recherche 1m** arrière / avant |
| <kbd>↑</kbd> / <kbd>↓</kbd> | **Volume** augmenter / diminuer (aussi <kbd>k</kbd> / <kbd>j</kbd>) |
| <kbd>1</kbd> - <kbd>9</kbd> | **Sauter** au pourcentage (10% - 90%) |
| <kbd>+</kbd> / <kbd>-</kbd> | **Vitesse** augmenter / diminuer |
| <kbd>0</kbd> | **Réinitialiser** la vitesse de lecture à 1.0x |
| <kbd>l</kbd> | **Basculer le mode boucle** à la volée |
| <kbd>m</kbd> | **Basculer le mode muet** |
| <kbd>q</kbd> / <kbd>ctrl+c</kbd> | **Quitter** la session |

---

## 🎥 Exemples de Téléchargement

### Assistant Interactif
Exécutez simplement la commande et suivez les invites tactiques pour sélectionner les flux, les formats et les options de métadonnées :
```bash
msc --download
```

### Préréglages Rapides
Téléchargez des versions de haute qualité directement sans l'assistant :
```bash
# MP3 haute qualité (320kbps)
msc --download=audio "https://youtube.com/watch?v=..."

# Vidéo meilleure qualité (MP4)
msc --download=video "https://youtube.com/watch?v=..."
```

---

## 🗑️ Désinstallation

### Windows
- Supprimez le dossier : `C:\Users\<Utilisateur>\.fireflylabs\firemusic`
- (Optionnel) Supprimez le dossier de votre PATH utilisateur dans les variables d'environnement.

### Linux / macOS
- Supprimez le binaire : `rm $(which msc)`
- (Si installé via cargo) : `cargo uninstall firemusic`

---

## 📜 Licence

Ce projet est sous licence **Apache 2.0**. Consultez le fichier `LICENSE` pour plus de détails.

Copyright © 2026-Present Firefly Labs
