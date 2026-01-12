use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImagePlaceholder {
    /// Alt text from markdown ![alt](path)
    pub alt_text: String,
    /// Relative path from the markdown file
    pub relative_path: String,
    /// Absolute path resolved from markdown file location
    pub absolute_path: PathBuf,
    /// Line number in the markdown file (1-indexed)
    pub line_number: usize,
    /// Whether the image file already exists
    pub exists: bool,
}

/// Parse a markdown file and extract all image placeholders
pub fn parse_markdown(markdown_path: &Path) -> Result<Vec<ImagePlaceholder>, String> {
    let content = fs::read_to_string(markdown_path)
        .map_err(|e| format!("Failed to read markdown file: {}", e))?;

    let markdown_dir = markdown_path
        .parent()
        .ok_or_else(|| "Invalid markdown path".to_string())?;

    // Pattern: ![alt text](path/to/image.ext)
    let re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)")
        .map_err(|e| format!("Regex error: {}", e))?;

    let mut placeholders = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        for cap in re.captures_iter(line) {
            let alt_text = cap.get(1).map_or("", |m| m.as_str()).to_string();
            let relative_path = cap.get(2).map_or("", |m| m.as_str()).to_string();

            // Skip URLs (http://, https://, data:)
            if relative_path.starts_with("http://")
                || relative_path.starts_with("https://")
                || relative_path.starts_with("data:")
            {
                continue;
            }

            let absolute_path = markdown_dir.join(&relative_path);
            let exists = absolute_path.exists();

            placeholders.push(ImagePlaceholder {
                alt_text,
                relative_path,
                absolute_path,
                line_number: line_idx + 1,
                exists,
            });
        }
    }

    Ok(placeholders)
}

/// Get only the placeholders that don't have existing images
pub fn get_unfilled_placeholders(markdown_path: &Path) -> Result<Vec<ImagePlaceholder>, String> {
    let all = parse_markdown(markdown_path)?;
    Ok(all.into_iter().filter(|p| !p.exists).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_markdown() {
        let content = r#"# Test Document

![Hero Image](images/hero.png)

Some text here.

![Screenshot](screenshots/step1.png)

![External](https://example.com/image.png)
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let placeholders = parse_markdown(file.path()).unwrap();

        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0].relative_path, "images/hero.png");
        assert_eq!(placeholders[0].alt_text, "Hero Image");
        assert_eq!(placeholders[1].relative_path, "screenshots/step1.png");
    }
}
