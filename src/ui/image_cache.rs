//! Image caching system for async image loading and texture management.
//!
//! This module provides thread-safe image caching with background loading
//! to avoid blocking the UI while fetching remote images.

use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;

/// Thread-safe image cache manager.
///
/// Handles async loading of images from URLs and caches the resulting
/// textures for efficient reuse.
#[derive(Clone)]
pub struct ImageCache {
    /// Cached textures by URL
    cache: Arc<Mutex<HashMap<String, egui::TextureHandle>>>,
    
    /// URLs currently being loaded
    loading: Arc<Mutex<HashSet<String>>>,
}

impl ImageCache {
    /// Creates a new empty image cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            loading: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Checks if any images are currently loading
    pub fn is_loading(&self) -> bool {
        let loading = self.loading.lock().unwrap();
        !loading.is_empty()
    }
    
    /// Gets a cached texture by URL if available
    pub fn get(&self, url: &str) -> Option<egui::TextureHandle> {
        let cache = self.cache.lock().unwrap();
        cache.get(url).cloned()
    }
    
    /// Starts loading an image from URL in the background
    ///
    /// Does nothing if the image is already cached or loading.
    pub fn load(&self, ctx: &egui::Context, url: String) {
        // Skip empty URLs
        if url.is_empty() {
            return;
        }
        
        // Check if already in cache
        {
            let cache = self.cache.lock().unwrap();
            if cache.contains_key(&url) {
                return;
            }
        }
        
        // Check if already loading
        {
            let mut loading = self.loading.lock().unwrap();
            if loading.contains(&url) {
                return;
            }
            loading.insert(url.clone());
        }
        
        let cache = self.cache.clone();
        let loading = self.loading.clone();
        let ctx = ctx.clone();
        
        thread::spawn(move || {
            // Download image
            if let Ok(response) = reqwest::blocking::get(&url) {
                if let Ok(bytes) = response.bytes() {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        let size = [image.width() as _, image.height() as _];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        );
                        
                        // Store in cache
                        let mut cache_guard = cache.lock().unwrap();
                        let texture = ctx.load_texture(
                            &url,
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        cache_guard.insert(url.clone(), texture);
                        
                        // Request repaint after texture is loaded
                        ctx.request_repaint();
                    }
                }
            }
            
            // Remove from loading set
            let mut loading_guard = loading.lock().unwrap();
            loading_guard.remove(&url);
        });
    }
    
    /// Clears all cached images
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}
