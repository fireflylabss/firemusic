#!/bin/bash

# 🔥 Fire Music Installer for Linux
# Developed by Firefly Labs

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔥 Fire Music - Installing...${NC}"

# 1. Check Dependencies
echo -e "🔍 Checking system dependencies..."

if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl is not installed.${NC}"
    exit 1
fi

if ! command -v mpv &> /dev/null; then
    echo -e "${YELLOW}⚠️ mpv not found. Attempting to install libmpv-dev...${NC}"
    sudo apt update && sudo apt install -y libmpv-dev mpv
fi

if ! command -v yt-dlp &> /dev/null; then
    echo -e "${YELLOW}⚠️ yt-dlp not found. Installing...${NC}"
    sudo apt install -y yt-dlp
fi

# 2. Check/Install Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}🦀 Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo -e "${GREEN}✅ Rust/Cargo detected.${NC}"
fi

# 3. Setup Directories
INSTALL_DIR="$HOME/.fireflylabs/firemusic"
echo -e "📁 Setting up directory: ${INSTALL_DIR}"
mkdir -p "$INSTALL_DIR"

# 4. Build and Install
echo -e "🚀 Building Fire Music..."
cargo build --release

# Copy binaries to internal dir
cp target/release/firemusic "$INSTALL_DIR/"
cp target/release/msc "$INSTALL_DIR/"
cp target/release/frmsc "$INSTALL_DIR/"

# 5. Link to System Path
echo -e "🔗 Linking to ~/.cargo/bin..."
mkdir -p "$HOME/.cargo/bin"
ln -sf "$INSTALL_DIR/firemusic" "$HOME/.cargo/bin/firemusic"
ln -sf "$INSTALL_DIR/msc" "$HOME/.cargo/bin/msc"
ln -sf "$INSTALL_DIR/frmsc" "$HOME/.cargo/bin/frmsc"

echo -e "${GREEN}✅ Fire Music installed successfully!${NC}"
echo -e "Try running: ${YELLOW}msc --help${NC}"
