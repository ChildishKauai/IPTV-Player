# Quick Start Development Guide

## Project Structure (Recommended)

\\\
src/
 main.rs              # Entry point and main app
 api/
    mod.rs           # API module exports
    xtream.rs        # Xtream Codes API client
    m3u.rs           # M3U parser
 models/
    mod.rs           # Data models exports
    channel.rs       # Channel struct
    category.rs      # Category struct
    config.rs        # App configuration
 ui/
    mod.rs           # UI module exports
    login.rs         # Login dialog
    sidebar.rs       # Category sidebar
    content.rs       # Channel grid
 player/
     mod.rs           # Player module exports
     mpv.rs           # MPV integration
\\\

## Running the Application

\\\ash
# Development mode (fast compile, includes debug info)
cargo run

# Release mode (optimized, slower compile, faster runtime)
cargo run --release

# Build only (no run)
cargo build
cargo build --release

# Check for errors without building
cargo check

# Format code
cargo fmt

# Run tests
cargo test
\\\

## Useful Commands

\\\ash
# Add a new dependency
cargo add <package-name>

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# View dependency tree
cargo tree

# Run with verbose output
cargo run -v
\\\

## egui Development Tips

### Immediate Mode GUI
- UI is rebuilt every frame (60fps)
- State lives in the app struct
- Use \ui.ctx().request_repaint()\ to trigger redraws

### Common Patterns

\\\ust
// Text input
ui.text_edit_singleline(&mut self.text);

// Button
if ui.button("Click me").clicked() {
    // Handle click
}

// Label
ui.label("Hello World");

// Horizontal layout
ui.horizontal(|ui| {
    ui.label("Name:");
    ui.text_edit_singleline(&mut self.name);
});

// Vertical layout (default)
ui.vertical(|ui| {
    ui.label("Line 1");
    ui.label("Line 2");
});

// Separator
ui.separator();

// Scroll area
egui::ScrollArea::vertical().show(ui, |ui| {
    // Content
});

// Side panel
egui::SidePanel::left("sidebar").show(ctx, |ui| {
    // Sidebar content
});

// Central panel
egui::CentralPanel::default().show(ctx, |ui| {
    // Main content
});
\\\

## Async Operations

\\\ust
// Spawn async task
let handle = tokio::spawn(async move {
    // Async work
});

// In egui context, use channels
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    // Background work
    tx.send(result).unwrap();
});

// Check for results in update()
if let Ok(result) = self.rx.try_recv() {
    // Handle result
}
\\\

## Video Player Integration Options

### Option 1: libmpv-rs (Recommended)
- Full control over mpv
- Best subtitle support
- Add to Cargo.toml: \libmpv = "2.0"\

### Option 2: mpv command
- Spawn mpv as subprocess
- Less control but simpler
- Use \std::process::Command\

### Option 3: gstreamer-rs
- More complex but powerful
- Better for custom pipelines
- Add to Cargo.toml: \gstreamer = "0.21"\

## Next Implementation Steps

1. **Create data models** (models/channel.rs, models/category.rs)
2. **Implement API client** (api/xtream.rs)
3. **Build UI layout** (ui/sidebar.rs, ui/content.rs)
4. **Add video player** (player/mpv.rs)
5. **Persist configuration** (models/config.rs)

## Common Issues & Solutions

### Issue: Compile errors about lifetimes
**Solution**: Use owned types (String, Vec) instead of references in structs

### Issue: Can't mutate data in egui callbacks
**Solution**: Use interior mutability (Arc<Mutex<T>>) or message passing

### Issue: Video player window embedding
**Solution**: Start with separate mpv window, then look into window embedding

### Issue: Async in egui
**Solution**: Use channels or poll-based approach, don't block UI thread

## Resources

- egui docs: https://docs.rs/egui
- Rust book: https://doc.rust-lang.org/book/
- Async book: https://rust-lang.github.io/async-book/
- tokio docs: https://docs.rs/tokio
