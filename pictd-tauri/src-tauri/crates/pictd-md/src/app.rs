use anyhow::anyhow;
use image::RgbaImage;
use pictd_core::{get_unfilled_placeholders, save_image_to_path, ImagePlaceholder};
use std::path::{Path, PathBuf};

pub struct App {
    pub markdown_path: PathBuf,
    pub placeholders: Vec<ImagePlaceholder>,
    pub selected_index: usize,
    pub clipboard_image: Option<RgbaImage>,
    pub clipboard_dimensions: Option<(u32, u32)>,
    pub last_image_hash: Option<u64>,
    pub status_message: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(markdown_path: &Path) -> anyhow::Result<Self> {
        let placeholders = get_unfilled_placeholders(markdown_path)
            .map_err(|e| anyhow!("Failed to parse markdown: {}", e))?;

        if placeholders.is_empty() {
            anyhow::bail!("No unfilled image placeholders found in the markdown file");
        }

        Ok(Self {
            markdown_path: markdown_path.to_path_buf(),
            placeholders,
            selected_index: 0,
            clipboard_image: None,
            clipboard_dimensions: None,
            last_image_hash: None,
            status_message: "Waiting for clipboard image...".to_string(),
            should_quit: false,
        })
    }

    pub fn select_next(&mut self) {
        if !self.placeholders.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.placeholders.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.placeholders.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.placeholders.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn check_clipboard(&mut self) {
        if let Some(img) = pictd_core::clipboard::get_clipboard_image() {
            let hash = pictd_core::clipboard::simple_hash(&img);

            if self.last_image_hash != Some(hash) {
                self.last_image_hash = Some(hash);
                self.clipboard_dimensions = Some((img.width(), img.height()));
                self.clipboard_image = Some(img);
                self.status_message = format!(
                    "Image ready: {}x{} - Press Enter to save",
                    self.clipboard_dimensions.unwrap().0,
                    self.clipboard_dimensions.unwrap().1
                );
            }
        }
    }

    pub fn save_to_selected(&mut self) -> anyhow::Result<()> {
        let Some(image) = self.clipboard_image.take() else {
            self.status_message = "No image in clipboard!".to_string();
            return Ok(());
        };

        if self.placeholders.is_empty() {
            self.status_message = "No more placeholders!".to_string();
            return Ok(());
        }

        let placeholder = &self.placeholders[self.selected_index];
        let target_path = &placeholder.absolute_path;

        match save_image_to_path(&image, target_path) {
            Ok(info) => {
                self.status_message = format!("Saved: {}", info.filename);

                // Remove the saved placeholder
                self.placeholders.remove(self.selected_index);

                // Adjust selection index if needed
                if !self.placeholders.is_empty() {
                    if self.selected_index >= self.placeholders.len() {
                        self.selected_index = self.placeholders.len() - 1;
                    }
                }

                // Clear clipboard state so user needs new image
                self.clipboard_dimensions = None;

                // Check if all done
                if self.placeholders.is_empty() {
                    self.status_message = "All placeholders filled! Press q to quit.".to_string();
                }
            }
            Err(e) => {
                self.status_message = format!("Error saving: {}", e);
                // Put the image back
                self.clipboard_image = Some(image);
            }
        }

        Ok(())
    }

    pub fn remaining_count(&self) -> usize {
        self.placeholders.len()
    }

    pub fn all_done(&self) -> bool {
        self.placeholders.is_empty()
    }
}
