const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let isMonitoring = true;

async function init() {
  // Load settings
  const settings = await invoke('get_settings');
  document.getElementById('save-dir').value = settings.save_dir;
  isMonitoring = settings.is_monitoring;
  updateStatusUI();

  // Load existing images
  await loadImages();

  // Listen for new images
  await listen('image-saved', (event) => {
    addImageCard(event.payload, true);
    hideEmptyMessage();
  });

  // Toggle button
  document.getElementById('toggle-btn').addEventListener('click', toggleMonitoring);
}

async function loadImages() {
  const images = await invoke('get_saved_images');
  const grid = document.getElementById('image-grid');

  if (images.length > 0) {
    hideEmptyMessage();
    images.forEach(img => addImageCard(img, false));
  }
}

function addImageCard(imageInfo, isNew) {
  const grid = document.getElementById('image-grid');

  const card = document.createElement('div');
  card.className = 'image-card' + (isNew ? ' new' : '');
  card.onclick = () => invoke('open_image', { path: imageInfo.path });

  card.innerHTML = `
    <img src="data:image/png;base64,${imageInfo.thumbnail}" alt="${imageInfo.filename}">
    <div class="info">
      <div class="filename">${imageInfo.filename}</div>
      <div class="dimensions">${imageInfo.width} x ${imageInfo.height}</div>
    </div>
  `;

  if (isNew) {
    grid.insertBefore(card, grid.firstChild);
  } else {
    grid.appendChild(card);
  }
}

function hideEmptyMessage() {
  const msg = document.getElementById('empty-message');
  if (msg) msg.style.display = 'none';
}

async function toggleMonitoring() {
  if (isMonitoring) {
    await invoke('stop_monitoring');
    isMonitoring = false;
  } else {
    await invoke('start_monitoring');
    isMonitoring = true;
  }
  updateStatusUI();
}

function updateStatusUI() {
  const indicator = document.getElementById('status-indicator');
  const text = document.getElementById('status-text');
  const btn = document.getElementById('toggle-btn');

  if (isMonitoring) {
    indicator.classList.remove('paused');
    text.textContent = 'Monitoring';
    btn.textContent = 'Pause';
  } else {
    indicator.classList.add('paused');
    text.textContent = 'Paused';
    btn.textContent = 'Resume';
  }
}

document.addEventListener('DOMContentLoaded', init);
