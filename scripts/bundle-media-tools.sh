#!/bin/bash
# Steam Deck Media Tools Bundler
# This script bundles ffmpeg, vlc, and mpv with the IPTV Player for Steam Deck
# Usage: ./bundle-media-tools.sh [output-dir]

set -e

OUTPUT_DIR="${1:-.}"
MEDIA_TOOLS_DIR="${OUTPUT_DIR}/media-tools"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on Steam Deck
check_steam_deck() {
    if grep -qi "SteamOS" /etc/os-release 2>/dev/null || [ -n "$SteamDeck" ]; then
        log_info "Running on Steam Deck (SteamOS detected)"
        return 0
    else
        log_warn "Not running on Steam Deck, but proceeding with bundling for Steam Deck compatibility"
        return 0
    fi
}

# Create bundled media tools directory
create_bundle_dir() {
    mkdir -p "$MEDIA_TOOLS_DIR"
    log_info "Created bundled media tools directory: $MEDIA_TOOLS_DIR"
}

# Function to find and copy a tool
bundle_tool() {
    local tool=$1
    local found=0
    
    log_info "Searching for $tool..."
    
    # Check common Steam Deck tool paths and system paths
    local search_paths=(
        "/usr/bin/$tool"
        "/usr/local/bin/$tool"
        "/opt/SteamOS/$tool"
        "/app/bin/$tool"
    )
    
    # Also try system find if available
    if command -v which &>/dev/null; then
        local sys_path=$(which "$tool" 2>/dev/null || echo "")
        if [ -n "$sys_path" ]; then
            search_paths=("$sys_path" "${search_paths[@]}")
        fi
    fi
    
    for path in "${search_paths[@]}"; do
        if [ -f "$path" ] && [ -x "$path" ]; then
            log_info "Found $tool at: $path"
            cp "$path" "$MEDIA_TOOLS_DIR/$tool"
            chmod +x "$MEDIA_TOOLS_DIR/$tool"
            log_info "Bundled $tool successfully"
            found=1
            break
        fi
    done
    
    if [ $found -eq 0 ]; then
        log_error "$tool not found in system"
        return 1
    fi
    
    return 0
}

# Bundle all media tools
bundle_all_tools() {
    local failed=0
    
    for tool in ffmpeg vlc mpv; do
        if ! bundle_tool "$tool"; then
            log_warn "Failed to bundle $tool - it may not be installed"
            ((failed++))
        fi
    done
    
    if [ $failed -eq 3 ]; then
        log_error "Could not find any media tools to bundle"
        log_info "Install ffmpeg, vlc, and/or mpv using: sudo steamos-readonly disable && sudo pacman -S ffmpeg vlc mpv && sudo steamos-readonly enable"
        return 1
    fi
    
    return 0
}

# Copy shared libraries dependencies (optional, advanced)
bundle_dependencies() {
    log_info "Bundling library dependencies..."
    
    local libs_dir="$MEDIA_TOOLS_DIR/lib"
    mkdir -p "$libs_dir"
    
    # For each bundled tool, find and copy critical dependencies
    for tool in ffmpeg vlc mpv; do
        if [ -f "$MEDIA_TOOLS_DIR/$tool" ]; then
            log_info "Analyzing dependencies for $tool..."
            
            # This is optional - only needed if tools don't link properly
            # ldd $MEDIA_TOOLS_DIR/$tool | grep "=>" | awk '{print $3}' | while read lib; do
            #     if [ -n "$lib" ] && [ -f "$lib" ]; then
            #         cp "$lib" "$libs_dir/" 2>/dev/null || true
            #     fi
            # done
        fi
    done
    
    log_info "Dependency bundling complete (note: some dependencies may be handled by Steam Deck system)"
}

# Create wrapper script for easy execution
create_wrapper_script() {
    local wrapper="$MEDIA_TOOLS_DIR/run-media-tool.sh"
    
    cat > "$wrapper" << 'EOF'
#!/bin/bash
# Wrapper to run bundled media tools with proper environment
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOL="${1:?Usage: $0 <tool> [args...]}"
shift

# Set library path if lib directory exists
if [ -d "$SCRIPT_DIR/lib" ]; then
    export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"
fi

# Execute the tool
exec "$SCRIPT_DIR/$TOOL" "$@"
EOF
    
    chmod +x "$wrapper"
    log_info "Created wrapper script: $wrapper"
}

# Create README for bundled tools
create_readme() {
    local readme="$MEDIA_TOOLS_DIR/README.md"
    
    cat > "$readme" << 'EOF'
# Bundled Media Tools for IPTV Player (Steam Deck)

This directory contains media tools bundled with the IPTV Player for Steam Deck.

## Included Tools

- **ffmpeg** - Video/audio encoding and transcoding
- **vlc** - VLC media player (if bundled)
- **mpv** - MPV video player (if bundled)

## Usage

These tools are automatically discovered and used by the IPTV Player when available.

To manually run a tool:
```bash
./run-media-tool.sh ffmpeg -version
./run-media-tool.sh vlc --version
./run-media-tool.sh mpv --version
```

## Environment

These bundled tools are optimized for Steam Deck (SteamOS) and may require specific runtime configurations.

## License

These tools are subject to their respective licenses:
- ffmpeg: LGPL/GPL
- vlc: GPL
- mpv: GPL

See the tool documentation for details.
EOF
    
    log_info "Created README: $readme"
}

# Main execution
main() {
    log_info "Steam Deck Media Tools Bundler"
    log_info "================================"
    log_info ""
    
    check_steam_deck
    create_bundle_dir
    
    if bundle_all_tools; then
        bundle_dependencies
        create_wrapper_script
        create_readme
        
        log_info ""
        log_info "✓ Bundling complete!"
        log_info "Media tools bundled in: $MEDIA_TOOLS_DIR"
        log_info ""
        log_info "Next steps:"
        log_info "1. Build with: cargo build --release --features bundle-media-tools"
        log_info "2. Copy the media-tools directory with your executable"
        return 0
    else
        log_error "Bundling failed!"
        return 1
    fi
}

main "$@"
