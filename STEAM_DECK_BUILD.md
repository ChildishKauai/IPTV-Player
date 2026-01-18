# Steam Deck Build & Bundling Guide

This guide explains how to build the IPTV Player for Steam Deck with bundled media tools (ffmpeg, vlc, mpv).

## Quick Start

### 1. On Your Development Machine (Windows/Linux/Mac)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Linux target for cross-compilation
rustup target add x86_64-unknown-linux-gnu

# Clone and build the project
git clone <repo>
cd iptv-player-rust
cargo build --release --target x86_64-unknown-linux-gnu
```

### 2. On Steam Deck (SteamOS)

```bash
# Enter Desktop Mode (press Steam + X, or use Quick Access Menu)

# Open Terminal
# Install Rust and build tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install media tools
sudo steamos-readonly disable
sudo pacman -S ffmpeg vlc mpv pkg-config
sudo steamos-readonly enable

# Build with bundled media tools
git clone <repo>
cd iptv-player-rust
./bundle-media-tools.sh
cargo build --release --features bundle-media-tools

# Your executable will be in: target/release/iptv-player
# Media tools will be in: media-tools/
```

## Detailed Setup

### Steam Deck Prerequisites

Steam Deck runs **SteamOS** (based on Arch Linux). To prepare for building:

1. **Disable Read-Only Filesystem:**
   ```bash
   sudo steamos-readonly disable
   ```

2. **Update Package Manager:**
   ```bash
   sudo pacman -Sy
   ```

3. **Install Rust (if needed):**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

4. **Install Required Packages:**
   ```bash
   # Core build tools
   sudo pacman -S base-devel
   
   # Media tools for bundling
   sudo pacman -S ffmpeg vlc mpv
   
   # Optional: development libraries
   sudo pacman -S pkg-config
   ```

5. **Re-enable Read-Only Filesystem:**
   ```bash
   sudo steamos-readonly enable
   ```

### Building the Project

#### Option A: Build Directly on Steam Deck (Recommended)

```bash
# Navigate to project directory
cd /path/to/iptv-player-rust

# Bundle media tools into the build
./bundle-media-tools.sh

# Build with bundled media tools feature
cargo build --release --features bundle-media-tools

# Built executable: target/release/iptv-player
```

#### Option B: Cross-Compile from Linux/macOS/Windows

```bash
# On your development machine

# Install the x86_64-unknown-linux-gnu target
rustup target add x86_64-unknown-linux-gnu

# Build for Steam Deck
cargo build --release --target x86_64-unknown-linux-gnu --features bundle-media-tools

# Output: target/x86_64-unknown-linux-gnu/release/iptv-player
```

**Note:** When cross-compiling, you'll need to manually copy media tools to the `media-tools/` directory on the Steam Deck.

## Bundled Media Tools

### Why Bundle?

- **Guaranteed compatibility** - Tools are built for SteamOS
- **Simplified deployment** - No need to install separate packages
- **Consistent experience** - Same versions across all installations

### What's Bundled

| Tool | Purpose | Feature |
|------|---------|---------|
| **ffmpeg** | Video transcoding & encoding | Always bundled |
| **vlc** | Fallback media player | Optional |
| **mpv** | High-quality video playback | Optional |

### File Structure

After bundling, your release will look like:

```
iptv-player/
├── iptv-player (executable)
└── media-tools/
    ├── ffmpeg
    ├── vlc (optional)
    ├── mpv (optional)
    ├── lib/ (optional, dependencies)
    ├── run-media-tool.sh (wrapper script)
    └── README.md
```

### Using Bundled Tools in Code

The `MediaToolsManager` in `src/media_tools.rs` automatically detects bundled tools:

```rust
use crate::media_tools::{MediaToolsManager, MediaTool};

let manager = MediaToolsManager::new();

// Check availability
if manager.is_available(MediaTool::FFmpeg) {
    println!("FFmpeg is available");
}

// Execute tool
let output = manager.execute(MediaTool::FFmpeg, &["-version"])?;
```

To use in UI, register the manager in your app state:

```rust
pub struct IPTVPlayerApp {
    #[serde(skip)]
    media_tools: MediaToolsManager,
    // ... other fields
}

impl IPTVPlayerApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let media_tools = MediaToolsManager::new();
        
        if media_tools.is_bundled_build() {
            eprintln!("Running bundled build - media tools available");
        }
        
        Self {
            media_tools,
            // ... other initialization
        }
    }
}
```

## Build Configurations

### Standard Build (Uses System Tools)

```bash
cargo build --release
```

- Uses system ffmpeg/vlc/mpv (or system PATH)
- Smaller binary size
- Requires user to install tools separately

### Bundled Build (Steam Deck Ready)

```bash
cargo build --release --features bundle-media-tools
```

- Includes media tools binary
- Enables bundling feature
- Larger binary size (~300-500MB depending on tools)

### Release Build

```bash
cargo build --release --features bundle-media-tools --target x86_64-unknown-linux-gnu
strip target/x86_64-unknown-linux-gnu/release/iptv-player
```

## Troubleshooting

### "Media tool not found" Error

If bundling fails:

1. **Verify tools are installed:**
   ```bash
   which ffmpeg vlc mpv
   ```

2. **Run bundler script:**
   ```bash
   ./bundle-media-tools.sh target/release/
   ```

3. **Check bundled directory:**
   ```bash
   ls -la media-tools/
   ```

### Build Fails on Steam Deck

**Issue:** Rust toolchain not found

**Solution:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Issue:** Missing dependencies

**Solution:**
```bash
sudo pacman -S base-devel pkg-config
```

### Tools Not Detected at Runtime

**Issue:** Bundled tools not found despite being bundled

**Solution:**
```bash
# Verify tools are executable
chmod +x media-tools/ffmpeg media-tools/vlc media-tools/mpv

# Check library dependencies
ldd media-tools/ffmpeg

# Use wrapper script
./media-tools/run-media-tool.sh ffmpeg -version
```

## Deployment to Steam Deck

### Method 1: Via USB Drive

1. **On your development machine:**
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu --features bundle-media-tools
   mkdir -p /mnt/usb/iptv-player
   cp target/x86_64-unknown-linux-gnu/release/iptv-player /mnt/usb/iptv-player/
   cp -r media-tools /mnt/usb/iptv-player/
   ```

2. **On Steam Deck:**
   ```bash
   mkdir -p ~/Applications/iptv-player
   cp -r /mnt/usb/iptv-player/* ~/Applications/iptv-player/
   chmod +x ~/Applications/iptv-player/iptv-player
   ```

### Method 2: Via SSH

1. **Build on development machine or Steam Deck**

2. **Transfer files:**
   ```bash
   scp -r ./iptv-player deck@steamdeck.local:~/Applications/
   ssh deck@steamdeck.local "chmod +x ~/Applications/iptv-player/iptv-player"
   ```

### Method 3: Native Build on Steam Deck

```bash
# Follow the "On Steam Deck (SteamOS)" section above
# Executable ready at: target/release/iptv-player
```

## Performance Optimization

### For Steam Deck

```bash
# Build with optimizations
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release --features bundle-media-tools

# Strip debug symbols for smaller binary
strip target/release/iptv-player
```

### Runtime Settings

Create `~/.iptv_player_config.json`:

```json
{
  "hardware_acceleration": true,
  "max_concurrent_streams": 2,
  "cache_size_mb": 500,
  "ui_scale": 1.2
}
```

## Testing on Steam Deck

### Emulation (Not Recommended for Final Testing)

For development, use QEMU:

```bash
# On Linux development machine
sudo pacman -S qemu-system-x86

# Run SteamOS in QEMU (advanced setup required)
```

### Remote Testing via SSH

```bash
# Build on Steam Deck
ssh deck@steamdeck.local << 'EOF'
cd ~/iptv-player-rust
./bundle-media-tools.sh
cargo build --release --features bundle-media-tools
./target/release/iptv-player
EOF
```

## Environment Variables

### Build-Time

- `STEAM_DECK_BUILD=1` - Force Steam Deck build mode
- `BUNDLE_MEDIA_TOOLS=1` - Enable tool bundling (set by build.rs)

### Runtime

- `SteamDeck=1` - Set by Steam when running on Steam Deck
- `RUST_LOG=debug` - Enable debug logging
- `LD_LIBRARY_PATH` - Automatically set by wrapper script

## Next Steps

1. ✅ Install prerequisites on Steam Deck
2. ✅ Run `bundle-media-tools.sh` to bundle media tools
3. ✅ Build with `cargo build --release --features bundle-media-tools`
4. ✅ Test the application
5. ✅ Deploy to Steam or distribute

## References

- [Steam Deck SteamOS Documentation](https://github.com/ValveSoftware/SteamOS)
- [Rust Embedded Guide](https://doc.rust-lang.org/beta/embedded-book/)
- [Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
