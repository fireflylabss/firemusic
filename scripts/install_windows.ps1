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

Clear-Host
Write-Host "`n"
Write-Host "  ███████╗██╗██████╗ ███████╗███╗   ███╗██╗   ██╗███████╗██╗ ██████╗ " -ForegroundColor Cyan
Write-Host "  ██╔════╝██║██╔══██╗██╔════╝████╗ ████║██║   ██║██╔════╝██║██╔════╝ " -ForegroundColor Cyan
Write-Host "  █████╗  ██║██████╔╝█████╗  ██╔████╔██║██║   ██║███████╗██║██║      " -ForegroundColor Cyan
Write-Host "  ██╔══╝  ██║██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║╚════██║██║██║      " -ForegroundColor Cyan
Write-Host "  ██║     ██║██║  ██║███████╗██║ ╚═╝ ██║╚██████╔╝███████║██║╚██████╗ " -ForegroundColor Cyan
Write-Host "  ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚══════╝╚═╝ ╚═════╝ " -ForegroundColor Cyan
Write-Host "`n"
Write-Host "  🔥 FIREMUSIC (msc) - Windows Tactical Installer" -ForegroundColor Yellow
Write-Host "  --------------------------------------------------"
Write-Host "  Base Path: $INSTALL_DIR" -ForegroundColor Gray

# 0. Check requirements
Write-Host "`n🔍 Checking requirements..."
$missing = $false

function Check-Command($cmd, $help) {
    if (!(Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Host "  ❌ Missing: $cmd" -ForegroundColor Red
        if ($help) { Write-Host "     $help" -ForegroundColor Yellow }
        return $true
    } else {
        Write-Host "  ✅ Found: $cmd" -ForegroundColor Green
        return $false
    }
}

$missing = $missing -or (Check-Command "git" "Required to clone the source code.")
$missing = $missing -or (Check-Command "cargo" "Required to compile the source. Install from: https://rustup.rs/")
$missing = $missing -or (Check-Command "tar" "Required to extract dependencies. Included in Windows 10/11.")

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
    Write-Host "  ✅ Directories ready." -ForegroundColor Green
} catch {
    Write-Host "  ❌ Failed to create directories: $_" -ForegroundColor Red
    exit 1
}

# 2. Download dependencies
Write-Host "`n🚀 Downloading yt-dlp.exe..."
Invoke-WebRequest -Uri $YTDLP_URL -OutFile (Join-Path $BIN_DIR "yt-dlp.exe")
Write-Host "  ✅ Downloaded yt-dlp." -ForegroundColor Green

Write-Host "`n📦 Downloading libmpv development files..."
Invoke-WebRequest -Uri $LIBMPV_URL -OutFile (Join-Path $TEMP_DIR "libmpv.7z")
Write-Host "  ✅ Downloaded libmpv package." -ForegroundColor Green

# 3. Extract libmpv
Write-Host "`n🛠️ Extracting libmpv (using system tar)..."
try {
    tar -xf (Join-Path $TEMP_DIR "libmpv.7z") -C $TEMP_DIR
    Write-Host "  ✅ Extraction complete." -ForegroundColor Green
} catch {
    Write-Host "  ❌ Extraction failed. Ensure 'tar' is functional or install 7zip." -ForegroundColor Red
    exit 1
}

# 4. Clone Source Code
Write-Host "`n🚀 Fetching source code..."
if (Test-Path $SRC_DIR) {
    Write-Host "  🔄 Updating existing source code..."
    Set-Location $SRC_DIR
    git pull | Out-Null
} else {
    Write-Host "  🚀 Cloning source code from GitHub..."
    git clone $REPO_URL $SRC_DIR | Out-Null
    Set-Location $SRC_DIR
}
Write-Host "  ✅ Source code ready." -ForegroundColor Green

# 5. Prepare environment and Compile
Write-Host "`n🏗️ Building firemusic... (this may take a minute)"

# Tell Cargo and the MSVC Linker exactly where to find the .lib and .h files
$env:LIB = "$TEMP_DIR;$TEMP_DIR\lib;$env:LIB"
$env:INCLUDE = "$TEMP_DIR\include;$env:INCLUDE"
$env:RUSTFLAGS = "-L native=$TEMP_DIR -L native=$TEMP_DIR\lib"

# Run cargo build without suppressing output so the user sees progress
cargo build --release --bins

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n  ❌ Build failed. Check the errors above." -ForegroundColor Red
    exit 1
}
Write-Host "  ✅ Build complete." -ForegroundColor Green

# 6. Move files to bin
Write-Host "`n🚚 Finalizing installation..."
$firemusicExe = Join-Path $SRC_DIR "target\release\firemusic.exe"
$mscExe = Join-Path $SRC_DIR "target\release\msc.exe"

Copy-Item $firemusicExe -Destination (Join-Path $BIN_DIR "firemusic.exe") -Force
Copy-Item $mscExe -Destination (Join-Path $BIN_DIR "msc.exe") -Force
Copy-Item $firemusicExe -Destination (Join-Path $BIN_DIR "frmsc.exe") -Force

# SMART DLL COPY: Look for any mpv-2.dll (with or without 'lib' prefix)
Write-Host "  🔍 Locating libmpv-2.dll..."
$dllFiles = Get-ChildItem -Path $TEMP_DIR -Filter "*mpv-2.dll" -Recurse
if ($dllFiles) {
    foreach ($file in $dllFiles) {
        Write-Host "  🚚 Copying $($file.Name) to bin..."
        Copy-Item $file.FullName -Destination $BIN_DIR -Force
    }
    Write-Host "  ✅ DLLs copied." -ForegroundColor Green
} else {
    Write-Host "  ⚠️ Warning: Could not find any mpv-2.dll in the package!" -ForegroundColor Red
}

# 7. Add to User PATH
Write-Host "`n🔗 Configuring environment variables..."
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($userPath -notlike "*$BIN_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    Write-Host "  ✅ Added $BIN_DIR to User PATH." -ForegroundColor Green
} else {
    Write-Host "  ℹ️ Path already configured." -ForegroundColor Gray
}

# Force path refresh in current session
$env:Path += ";$BIN_DIR"

# 8. Cleanup
Write-Host "`n🧹 Cleaning up temporary files..."
Set-Location $INSTALL_DIR
if (Test-Path $TEMP_DIR) {
    Remove-Item -Recurse -Force $TEMP_DIR
}

Write-Host "`n🔥 FIREMUSIC INSTALLED SUCCESSFULLY!" -ForegroundColor Yellow -BackgroundColor Black
Write-Host "  --------------------------------------------------"
Write-Host "  You can now type 'msc' in this terminal to start!"
Write-Host "  Location: $BIN_DIR"
Write-Host "  To uninstall, delete: $INSTALL_DIR"
Write-Host "  --------------------------------------------------"

# Final check
if (Test-Path (Join-Path $BIN_DIR "msc.exe")) {
    Write-Host "  🚀 Verification success: msc.exe found." -ForegroundColor Green
} else {
    Write-Host "  ⚠️ Error: msc.exe not found in bin folder!" -ForegroundColor Red
}
