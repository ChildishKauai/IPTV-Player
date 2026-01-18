use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // Enable Steam Deck bundling
    if is_steam_deck_build() {
        println!("cargo:rustc-env=BUNDLE_MEDIA_TOOLS=1");
        println!("cargo:rustc-env=STEAM_DECK_BUILD=1");
        
        // Set rpath for bundled binaries on Linux
        if target_os == "linux" {
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/media-tools");
        }
    }
    
    // Print rerun-if-changed to make cargo rerun build script if build config changes
    println!("cargo:rerun-if-env-changed=STEAM_DECK_BUILD");
    println!("cargo:rerun-if-env-changed=BUNDLE_MEDIA_TOOLS");
}

fn is_steam_deck_build() -> bool {
    // Check for explicit Steam Deck build flag
    if env::var("STEAM_DECK_BUILD").is_ok() {
        return true;
    }
    
    // Check for Steam Deck specific environment variable (set by Steam at runtime)
    if env::var("SteamDeck").is_ok() {
        return true;
    }
    
    // Check for Steam Deck specific HWDB/etc detection if needed
    false
}
