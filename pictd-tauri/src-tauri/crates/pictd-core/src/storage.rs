use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Local;
use image::RgbaImage;
use serde::Serialize;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Debug)]
pub struct ImageInfo {
    pub path: String,
    pub filename: String,
    pub timestamp: String,
    pub width: u32,
    pub height: u32,
    pub thumbnail: String, // base64 encoded
}

pub fn get_downloads_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Save image with auto-generated timestamp filename
pub fn save_image(image: &RgbaImage, save_dir: &str) -> Result<ImageInfo, String> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut filename = format!("{}.png", timestamp);
    let save_path = PathBuf::from(save_dir);

    // Ensure directory exists
    fs::create_dir_all(&save_path).map_err(|e| e.to_string())?;

    let mut filepath = save_path.join(&filename);

    // Handle duplicate timestamps
    let mut counter = 1;
    while filepath.exists() {
        filename = format!("{}_{}.png", timestamp, counter);
        filepath = save_path.join(&filename);
        counter += 1;
    }

    // Save the image
    image.save(&filepath).map_err(|e| e.to_string())?;

    // Generate thumbnail
    let thumbnail = generate_thumbnail(image)?;

    Ok(ImageInfo {
        path: filepath.to_string_lossy().to_string(),
        filename,
        timestamp,
        width: image.width(),
        height: image.height(),
        thumbnail,
    })
}

/// Save image to a specific target path (for markdown mode)
pub fn save_image_to_path(image: &RgbaImage, target_path: &Path) -> Result<ImageInfo, String> {
    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Save the image
    image.save(target_path).map_err(|e| e.to_string())?;

    let filename = target_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Generate thumbnail
    let thumbnail = generate_thumbnail(image)?;

    Ok(ImageInfo {
        path: target_path.to_string_lossy().to_string(),
        filename,
        timestamp,
        width: image.width(),
        height: image.height(),
        thumbnail,
    })
}

fn generate_thumbnail(image: &RgbaImage) -> Result<String, String> {
    let thumb = image::imageops::resize(image, 150, 150, image::imageops::FilterType::Triangle);

    let mut buf = Cursor::new(Vec::new());
    thumb
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(STANDARD.encode(buf.into_inner()))
}

pub fn list_saved_images(save_dir: &str) -> Vec<ImageInfo> {
    let save_path = PathBuf::from(save_dir);
    let mut images = Vec::new();

    if let Ok(entries) = fs::read_dir(&save_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "png") {
                if let Ok(img) = image::open(&path) {
                    let rgba = img.to_rgba8();
                    if let Ok(thumbnail) = generate_thumbnail(&rgba) {
                        let filename = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let timestamp = filename.trim_end_matches(".png").to_string();

                        images.push(ImageInfo {
                            path: path.to_string_lossy().to_string(),
                            filename,
                            timestamp,
                            width: rgba.width(),
                            height: rgba.height(),
                            thumbnail,
                        });
                    }
                }
            }
        }
    }

    // Sort by filename (timestamp) descending
    images.sort_by(|a, b| b.filename.cmp(&a.filename));
    images
}
