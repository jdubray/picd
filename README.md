# picd

A clipboard image saver for WSL. Monitors the Windows clipboard and automatically saves images to your Downloads folder with timestamp filenames.

## Requirements

- Windows with WSL (Windows Subsystem for Linux)
- Python 3
- Pillow library

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/jdubray/picd.git
   cd picd
   ```

2. Create and activate a virtual environment:
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   ```

3. Install dependencies:
   ```bash
   pip install Pillow
   ```

## Usage

Run the script:
```bash
python clipboard_saver.py
```

The script will:
- Monitor your clipboard every 0.5 seconds
- Automatically save any new images to your Downloads folder
- Name files with timestamps (e.g., `2026-01-11_12-30-45.png`)

Press `Ctrl+C` to stop.

## Configuration

Edit `clipboard_saver.py` to change:
- `DOWNLOADS_DIR` - Where images are saved
- `CHECK_INTERVAL` - How often to check the clipboard (in seconds)
