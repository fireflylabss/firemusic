#!/bin/bash

# 🔥 Fire Music Installer for macOS
# Developed by Firefly Labs

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔥 Fire Music - Installation Wizard (macOS)${NC}"

# 1. Strict Dependency Check
MISSING=0

check_cmd() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ Missing: $1${NC}"
        MISSING=1
    else
        echo -e "${GREEN}✅ Found: $1${NC}"
    fi
}

echo -e "🔍 Checking requirements..."
check_cmd "brew"
check_cmd "git"
check_cmd "cargo"
check_cmd "mpv"
check_cmd "yt-dlp"

if [ $MISSING -eq 1 ]; then
    echo -e "\n${RED}Installation aborted due to missing dependencies.${NC}"
    echo -e "Please install the following before running this script again:"
    echo -e " - ${YELLOW}Homebrew:${NC} /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    echo -e " - ${YELLOW}Rust/Cargo:${NC} curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo -e " - ${YELLOW}Dependencies via brew:${NC} brew install mpv yt-dlp git"
    exit 1
fi

# 2. Setup Directories
INSTALL_DIR="$HOME/.fireflylabs/firemusic"
TEMP_DIR=$(mktemp -d)
echo -e "\n📁 Setting up directory: ${INSTALL_DIR}"
mkdir -p "$INSTALL_DIR"

# 3. Clone and Build
echo -e "🚀 Fetching source code..."
git clone https://github.com/fireflylabss/firemusic.git "$TEMP_DIR"
cd "$TEMP_DIR"

echo -e "🚀 Building Fire Music (Release)..."
cargo build --release

# Copy binaries to internal dir
cp target/release/firemusic "$INSTALL_DIR/"
cp target/release/msc "$INSTALL_DIR/"
cp target/release/frmsc "$INSTALL_DIR/"

# 4. Link to System Path
echo -e "🔗 Linking to ~/.cargo/bin..."
mkdir -p "$HOME/.cargo/bin"
ln -sf "$INSTALL_DIR/firemusic" "$HOME/.cargo/bin/firemusic"
ln -sf "$INSTALL_DIR/msc" "$HOME/.cargo/bin/msc"
ln -sf "$INSTALL_DIR/frmsc" "$HOME/.cargo/bin/frmsc"

# 5. Cleanup
rm -rf "$TEMP_DIR"

echo -e "\n${GREEN}✅ Fire Music installed successfully!${NC}"
echo -e "Try running: ${YELLOW}msc --help${NC}"
