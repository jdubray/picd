"""
Clipboard Image Saver
Monitors clipboard and saves images to Downloads folder with timestamp names.
"""

import os
import subprocess
import time
from datetime import datetime
from io import BytesIO
from PIL import Image

DOWNLOADS_DIR = "/mnt/c/Users/jjdub/Downloads"
CHECK_INTERVAL = 0.5  # seconds

def get_clipboard_image():
    """Get image from Windows clipboard via PowerShell (works in WSL)."""
    try:
        # Use PowerShell to get clipboard image and output as PNG bytes
        ps_script = '''
Add-Type -AssemblyName System.Windows.Forms
$clip = [System.Windows.Forms.Clipboard]::GetImage()
if ($clip) {
    $ms = New-Object System.IO.MemoryStream
    $clip.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    [Convert]::ToBase64String($ms.ToArray())
}
'''
        result = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps_script],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0 and result.stdout.strip():
            import base64
            image_data = base64.b64decode(result.stdout.strip())
            return Image.open(BytesIO(image_data))
    except Exception:
        pass
    return None

def save_image(image):
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    filename = f"{timestamp}.png"
    filepath = os.path.join(DOWNLOADS_DIR, filename)

    # Handle duplicate timestamps
    counter = 1
    while os.path.exists(filepath):
        filename = f"{timestamp}_{counter}.png"
        filepath = os.path.join(DOWNLOADS_DIR, filename)
        counter += 1

    image.save(filepath, "PNG")
    print(f"Saved: {filename}")

def main():
    print(f"Monitoring clipboard for images...")
    print(f"Saving to: {DOWNLOADS_DIR}")
    print("Press Ctrl+C to stop.\n")

    last_image = None

    while True:
        image = get_clipboard_image()

        if image is not None and hasattr(image, 'tobytes'):
            # Check if it's a new image by comparing bytes
            current_bytes = image.tobytes()
            if last_image != current_bytes:
                save_image(image)
                last_image = current_bytes

        time.sleep(CHECK_INTERVAL)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nStopped.")
