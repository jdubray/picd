use arboard::Clipboard;
use image::{ImageBuffer, RgbaImage};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::storage;

pub struct ClipboardMonitor {
    running: Arc<AtomicBool>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, app_handle: AppHandle, save_dir: String) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        thread::spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to access clipboard: {}", e);
                    return;
                }
            };

            let mut last_image_hash: Option<u64> = None;

            while running.load(Ordering::SeqCst) {
                if let Ok(img_data) = clipboard.get_image() {
                    let rgba_image: RgbaImage = ImageBuffer::from_raw(
                        img_data.width as u32,
                        img_data.height as u32,
                        img_data.bytes.into_owned(),
                    )
                    .unwrap_or_else(|| ImageBuffer::new(1, 1));

                    let current_hash = simple_hash(&rgba_image);

                    if last_image_hash != Some(current_hash) {
                        last_image_hash = Some(current_hash);

                        match storage::save_image(&rgba_image, &save_dir) {
                            Ok(saved_info) => {
                                let _ = app_handle.emit("image-saved", &saved_info);
                            }
                            Err(e) => {
                                eprintln!("Failed to save image: {}", e);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(500));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

fn simple_hash(img: &RgbaImage) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    img.dimensions().hash(&mut hasher);

    // Sample pixels for faster hashing
    let (w, h) = img.dimensions();
    let step = ((w * h) / 1000).max(1) as usize;
    for (i, pixel) in img.pixels().enumerate() {
        if i % step == 0 {
            pixel.0.hash(&mut hasher);
        }
    }

    hasher.finish()
}
