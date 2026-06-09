#!/bin/bash

# 🔥 FireMusic Installer for Unix (Linux & macOS)
# Developed by Firefly Labs

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${YELLOW}🔥 FireMusic - Unix Tactical Installer${NC}"

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
check_cmd "curl"
check_cmd "git"
check_cmd "cargo"
check_cmd "mpv"
check_cmd "yt-dlp"

# Special check for libmpv headers (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [ -f /usr/include/mpv/client.h ] || [ -f /usr/local/include/mpv/client.h ]; then
        echo -e "${GREEN}✅ Found: libmpv development headers${NC}"
    else
        echo -e "${RED}❌ Missing: libmpv development headers (libmpv-dev)${NC}"
        MISSING=1
    fi
fi

if [ $MISSING -eq 1 ]; then
    echo -e "\n${RED}Installation aborted due to missing dependencies.${NC}"
    echo -e "Please install the following before running this script again:"
    echo -e " - ${YELLOW}Rust/Cargo:${NC} curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo -e " - ${YELLOW}Dependencies:${NC} sudo apt install libmpv-dev mpv yt-dlp git"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo -e " - ${YELLOW}Dependencies:${NC} brew install mpv yt-dlp git"
    fi
    exit 1
fi

# 2. Setup Directories
INSTALL_DIR="$HOME/.fireflylabs/firemusic"
SRC_DIR="$INSTALL_DIR/src"
BIN_DIR="$INSTALL_DIR/bin"

echo -e "\n📁 Setting up directory structure..."
mkdir -p "$BIN_DIR"
mkdir -p "$SRC_DIR"

# 3. Clone and Build
echo -e "🚀 Fetching source code..."
if [ -d "$SRC_DIR/.git" ]; then
    echo -e "🔄 Updating existing source code..."
    cd "$SRC_DIR"
    git pull
else
    echo -e "🚀 Cloning source code..."
    git clone https://github.com/fireflylabss/firemusic.git "$SRC_DIR"
    cd "$SRC_DIR"
fi

echo -e "🏗️ Building FireMusic (Release) with Cargo..."
cargo build --release --bins

# 4. Copy binaries
echo -e "🚚 Finalizing installation..."
cp target/release/firemusic "$BIN_DIR/firemusic"
cp target/release/msc "$BIN_DIR/msc"
cp target/release/firemusic "$BIN_DIR/frmsc"

# 5. Add to PATH
echo -e "🔗 Configuring PATH..."

add_to_path() {
    local shell_config="$1"
    if [ -f "$shell_config" ]; then
        if ! grep -q "$BIN_DIR" "$shell_config"; then
            echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$shell_config"
            echo -e "${GREEN}✅ Added $BIN_DIR to $shell_config${NC}"
        else
            echo -e "ℹ️ $BIN_DIR is already in $shell_config"
        fi
    fi
}

# Also symlink to ~/.cargo/bin as a fallback
mkdir -p "$HOME/.cargo/bin"
ln -sf "$BIN_DIR/firemusic" "$HOME/.cargo/bin/firemusic"
ln -sf "$BIN_DIR/msc" "$HOME/.cargo/bin/msc"
ln -sf "$BIN_DIR/frmsc" "$HOME/.cargo/bin/frmsc"

# Try to add directly to shell profiles
add_to_path "$HOME/.bashrc"
add_to_path "$HOME/.zshrc"

echo -e "\n${GREEN}✅ FireMusic installed successfully!${NC}"
echo -e "The binary is located at: ${CYAN}$BIN_DIR/msc${NC}"
echo -e "It has been symlinked to your ${CYAN}~/.cargo/bin${NC} directory."
echo -e "Please restart your terminal or run: ${YELLOW}source ~/.bashrc${NC} (or ~/.zshrc)"
echo -e "Try running: ${YELLOW}msc --help${NC}"
echo -e "To uninstall, simply delete the folder: ${YELLOW}$INSTALL_DIR${NC}"
