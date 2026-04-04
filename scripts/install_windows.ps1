# firemusic (msc) - Windows Tactical Installer
# This script installs firemusic in a tactical, isolated directory: $HOME\.fireflylabs\msc

$ErrorActionPreference = "Stop"

$INSTALL_DIR = "$HOME\.fireflylabs\firemusic"
$SRC_DIR = "$INSTALL_DIR\src"
$BIN_DIR = "$INSTALL_DIR\bin"
$TEMP_DIR = "$INSTALL_DIR\temp"

# URLs
$REPO_URL = "https://github.com/fireflylabss/firemusic.git"
$YTDLP_URL = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
$LIBMPV_URL = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260301/mpv-dev-x86_64-v3-20260301-git-05fac7f.7z"

Write-Host "🔥 firemusic (msc) - Windows Tactical Installer" -ForegroundColor Yellow

# 0. Check requirements
Write-Host "🔍 Checking requirements..."
$missing = $false

if (!(Get-Command "git" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Missing: git (Required to clone the source code)" -ForegroundColor Red
    $missing = $true
} else {
    Write-Host "✅ Found: git" -ForegroundColor Green
}

if (!(Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Missing: cargo/rust (Required to compile the source code)" -ForegroundColor Red
    Write-Host "👉 Install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    $missing = $true
} else {
    Write-Host "✅ Found: cargo" -ForegroundColor Green
}

if ($missing) {
    Write-Host "`n❌ Installation aborted due to missing dependencies." -ForegroundColor Red
    exit 1
}

# 1. Create directory structure
Write-Host "`n📁 Creating directory structure..."
New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $TEMP_DIR | Out-Null

# 2. Download dependencies
Write-Host "🚀 Downloading yt-dlp.exe..."
Invoke-WebRequest -Uri $YTDLP_URL -OutFile "$BIN_DIR\yt-dlp.exe"

Write-Host "📦 Downloading libmpv development files..."
Invoke-WebRequest -Uri $LIBMPV_URL -OutFile "$TEMP_DIR\libmpv.7z"

# 3. Extract libmpv
Write-Host "🛠️ Extracting libmpv (using system tar)..."
tar -xf "$TEMP_DIR\libmpv.7z" -C "$TEMP_DIR"

# 4. Clone Source Code
if (Test-Path $SRC_DIR) {
    Write-Host "🔄 Updating existing source code..."
    Set-Location $SRC_DIR
    git pull
} else {
    Write-Host "🚀 Cloning source code..."
    git clone $REPO_URL $SRC_DIR
    Set-Location $SRC_DIR
}

# 5. Prepare environment and Compile
Write-Host "🏗️ Building firemusic (msc) with Cargo..."
$env:MPV_LIB_PATH = "$TEMP_DIR"
$env:INCLUDE = "$TEMP_DIR\include"

cargo build --release

# 6. Move files to bin
Write-Host "🚚 Finalizing installation..."
Copy-Item "target\release\firemusic.exe" -Destination "$BIN_DIR\msc.exe" -Force
Copy-Item "$TEMP_DIR\mpv-2.dll" -Destination "$BIN_DIR\" -Force

# 7. Add to User PATH
Write-Host "🔗 Adding to User PATH..."
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$BIN_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$BIN_DIR", "User")
    $env:Path += ";$BIN_DIR"
    Write-Host "✅ Added $BIN_DIR to PATH." -ForegroundColor Green
} else {
    Write-Host "ℹ️ $BIN_DIR is already in PATH."
}

# 8. Cleanup
Write-Host "🧹 Cleaning up temporary files..."
Set-Location $INSTALL_DIR
Remove-Item -Recurse -Force $TEMP_DIR

Write-Host "`n✅ firemusic (msc) installed successfully!" -ForegroundColor Green
Write-Host "Restart your terminal and type 'msc' to start."
Write-Host "To uninstall, simply delete the folder: $INSTALL_DIR"
