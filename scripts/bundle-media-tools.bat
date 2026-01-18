@echo off
REM Steam Deck Media Tools Bundler (Windows batch version)
REM This creates a template for bundling media tools
REM Note: For actual bundling, run on Steam Deck or use bundle-media-tools.sh

setlocal enabledelayedexpansion
set OUTPUT_DIR=%1
if "!OUTPUT_DIR!"=="" set OUTPUT_DIR=.

set MEDIA_TOOLS_DIR=!OUTPUT_DIR!\media-tools

echo [INFO] Steam Deck Media Tools Bundler
echo [INFO] ================================
echo.
echo [WARN] This is a Windows helper script. For actual bundling on Steam Deck, use:
echo        ./bundle-media-tools.sh
echo.
echo [INFO] Creating media-tools directory structure...
if not exist "!MEDIA_TOOLS_DIR!" mkdir "!MEDIA_TOOLS_DIR!"

echo. > "!MEDIA_TOOLS_DIR!\README.md"
echo # Bundled Media Tools for IPTV Player ^(Steam Deck^) >> "!MEDIA_TOOLS_DIR!\README.md"
echo. >> "!MEDIA_TOOLS_DIR!\README.md"
echo This directory should contain bundled ffmpeg, vlc, and mpv binaries for Steam Deck. >> "!MEDIA_TOOLS_DIR!\README.md"
echo. >> "!MEDIA_TOOLS_DIR!\README.md"
echo ## Setup Instructions >> "!MEDIA_TOOLS_DIR!\README.md"
echo. >> "!MEDIA_TOOLS_DIR!\README.md"
echo On Steam Deck:^(SteamOS^): >> "!MEDIA_TOOLS_DIR!\README.md"
echo 1. Disable readonly: `sudo steamos-readonly disable` >> "!MEDIA_TOOLS_DIR!\README.md"
echo 2. Install tools: `sudo pacman -S ffmpeg vlc mpv` >> "!MEDIA_TOOLS_DIR!\README.md"
echo 3. Run bundler: `./bundle-media-tools.sh` >> "!MEDIA_TOOLS_DIR!\README.md"
echo 4. Enable readonly: `sudo steamos-readonly enable` >> "!MEDIA_TOOLS_DIR!\README.md"

echo [INFO] Created media-tools directory: !MEDIA_TOOLS_DIR!
echo [INFO] Run bundle-media-tools.sh on Steam Deck to populate with actual binaries
echo.
echo [INFO] To build with media tools bundling feature:
echo        cargo build --release --features bundle-media-tools
pause
