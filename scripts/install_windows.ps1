# firemusic (msc) - Windows Tactical Installer
# This script installs firemusic in a tactical, isolated directory: $HOME\.fireflylabs\firemusic

$ErrorActionPreference = "Stop"

# Use absolute paths based on HOME
$INSTALL_DIR = Join-Path $HOME ".fireflylabs\firemusic"
$SRC_DIR = Join-Path $INSTALL_DIR "src"
$BIN_DIR = Join-Path $INSTALL_DIR "bin"
$TEMP_DIR = Join-Path $INSTALL_DIR "temp"

# URLs
$REPO_URL = "https://github.com/fireflylabss/firemusic.git"
$YTDLP_URL = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
$LIBMPV_URL = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260301/mpv-dev-x86_64-v3-20260301-git-05fac7f.7z"

Write-Host "`n🔥 FIREMUSIC (msc) - Windows Tactical Installer" -ForegroundColor Yellow
Write-Host "--------------------------------------------------"
Write-Host "Base Path: $INSTALL_DIR"

# 0. Check requirements
Write-Host "`n🔍 Checking requirements..."
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
try {
    if (!(Test-Path $INSTALL_DIR)) { New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null }
    if (!(Test-Path $BIN_DIR)) { New-Item -ItemType Directory -Path $BIN_DIR -Force | Out-Null }
    if (!(Test-Path $TEMP_DIR)) { New-Item -ItemType Directory -Path $TEMP_DIR -Force | Out-Null }
    Write-Host "✅ Directories ready." -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to create directories: $_" -ForegroundColor Red
    exit 1
}

# 2. Download dependencies
Write-Host "`n🚀 Downloading yt-dlp.exe..."
Invoke-WebRequest -Uri $YTDLP_URL -OutFile (Join-Path $BIN_DIR "yt-dlp.exe")

Write-Host "📦 Downloading libmpv development files..."
Invoke-WebRequest -Uri $LIBMPV_URL -OutFile (Join-Path $TEMP_DIR "libmpv.7z")

# 3. Extract libmpv
Write-Host "🛠️ Extracting libmpv (using system tar)..."
tar -xf (Join-Path $TEMP_DIR "libmpv.7z") -C $TEMP_DIR

# 4. Clone Source Code
if (Test-Path $SRC_DIR) {
    Write-Host "`n🔄 Updating existing source code..."
    Set-Location $SRC_DIR
    git pull
} else {
    Write-Host "`n🚀 Cloning source code from GitHub..."
    git clone $REPO_URL $SRC_DIR
    Set-Location $SRC_DIR
}

# 5. Prepare environment and Compile
Write-Host "`n🏗️ Building firemusic (msc) with Cargo... (this may take a minute)"

# Tell Cargo and the MSVC Linker exactly where to find the .lib and .h files
$env:LIB = "$TEMP_DIR;$TEMP_DIR\lib;$env:LIB"
$env:INCLUDE = "$TEMP_DIR\include;$env:INCLUDE"
$env:RUSTFLAGS = "-L native=$TEMP_DIR -L native=$TEMP_DIR\lib"

cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n❌ Build failed. Please check the errors above." -ForegroundColor Red
    exit 1
}

# 6. Move files to bin
Write-Host "`n🚚 Finalizing installation..."
Copy-Item "target\release\firemusic.exe" -Destination (Join-Path $BIN_DIR "msc.exe") -Force
Copy-Item "target\release\firemusic.exe" -Destination (Join-Path $BIN_DIR "firemusic.exe") -Force
Copy-Item "target\release\firemusic.exe" -Destination (Join-Path $BIN_DIR "frmsc.exe") -Force

# Copy the DLL from wherever it was extracted
if (Test-Path (Join-Path $TEMP_DIR "mpv-2.dll")) {
    Copy-Item (Join-Path $TEMP_DIR "mpv-2.dll") -Destination $BIN_DIR -Force
} elseif (Test-Path (Join-Path $TEMP_DIR "lib\mpv-2.dll")) {
    Copy-Item (Join-Path $TEMP_DIR "lib\mpv-2.dll") -Destination $BIN_DIR -Force
}

# 7. Add to User PATH
Write-Host "🔗 Configuring environment variables..."
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($userPath -notlike "*$BIN_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    Write-Host "✅ Added $BIN_DIR to User PATH." -ForegroundColor Green
}

# Force path refresh in current session
$env:Path += ";$BIN_DIR"

# 8. Cleanup
Write-Host "🧹 Cleaning up temporary files..."
Set-Location $INSTALL_DIR
if (Test-Path $TEMP_DIR) {
    Remove-Item -Recurse -Force $TEMP_DIR
}

Write-Host "`n🔥 FIREMUSIC INSTALLED SUCCESSFULLY!" -ForegroundColor Yellow -BackgroundColor Black
Write-Host "--------------------------------------------------"
Write-Host "You can now type 'msc' in this terminal to start!"
Write-Host "Location: $BIN_DIR"
Write-Host "To uninstall, delete: $INSTALL_DIR"
Write-Host "--------------------------------------------------"

# Final check
if (Test-Path (Join-Path $BIN_DIR "msc.exe")) {
    Write-Host "🚀 Verification success: msc.exe found." -ForegroundColor Green
} else {
    Write-Host "⚠️ Warning: msc.exe not found in bin folder!" -ForegroundColor Red
}
