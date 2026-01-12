# pictd

A clipboard image saver with two modes:
- **GUI app** - System tray app that auto-saves clipboard images with timestamps
- **Markdown helper** - TUI tool for filling image placeholders in documentation

## pictd-md (Markdown Screenshot Helper)

Helps you create user manuals with screenshots. Parses a markdown file, finds image placeholders, and lets you fill them interactively from your clipboard.

### Installation

Requires Rust. Install from https://rustup.rs or:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build the tool:
```bash
cd pictd-tauri/src-tauri
cargo build --release -p pictd-md
```

Optionally copy to your PATH:
```bash
cp target/release/pictd-md ~/.local/bin/
```

### Usage

```bash
pictd-md /path/to/your-document.md
```

The TUI shows all unfilled image placeholders from your markdown:

```
┌─ pictd-md ─ /docs/guide.md (3 remaining) ─────┐
│ Image Placeholders:                            │
│                                                │
│   screenshots/intro.png           (line 5)    │
│ > screenshots/property-inspector.png (line 12)│
│   diagrams/flow.png               (line 28)   │
│                                                │
├────────────────────────────────────────────────┤
│ Clipboard: IMAGE READY 800x600                 │
│ ↑↓ navigate  Enter save  q quit                │
└────────────────────────────────────────────────┘
```

**Workflow:**
1. Copy a screenshot to your clipboard
2. Use ↑↓ arrows to select which placeholder to fill
3. Press Enter to save the image
4. Repeat until all placeholders are filled
5. Press q to quit

**Features:**
- Images are saved relative to the markdown file location
- Directories are created automatically
- Already-existing images are hidden from the list
- You can quit and resume later - only unfilled placeholders appear

### Markdown Format

The tool finds standard markdown image references:
```markdown
![Alt Text](path/to/image.png)
```

URLs (http://, https://) are ignored.

---

## pictd GUI App

Desktop app with system tray that monitors clipboard and auto-saves images.

### Installation

Requires Node.js and Rust.

```bash
cd pictd-tauri
npm install
npm run tauri build
```

### Usage

Run the app - it sits in your system tray and automatically saves any images you copy to your clipboard to your Downloads folder with timestamp filenames (e.g., `2026-01-11_12-30-45.png`).

---

## Legacy Python Version

A simple Python script for WSL environments.

### Requirements
- Python 3
- Pillow library

### Installation
```bash
python3 -m venv venv
source venv/bin/activate
pip install Pillow
```

### Usage
```bash
python clipboard_saver.py
```

Press `Ctrl+C` to stop.
