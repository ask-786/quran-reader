// Points the download links at the files in the latest release. Release asset
// names carry the version (Quran.Reader_0.2.12_amd64.deb), so a fixed
// releases/latest/download/<name> link would break on every release. Without
// JavaScript, or if the API call fails, every link still goes to the latest
// release page.

const REPO = 'ask-786/quran-reader';
const CACHE_KEY = 'latest-release';

const ASSETS = {
  exe: /_x64-setup\.exe$/,
  msi: /\.msi$/,
  dmg: /\.dmg$/,
  appimage: /\.AppImage$/,
  deb: /\.deb$/,
  rpm: /\.rpm$/,
};

const PRIMARY = {
  windows: { asset: 'exe', label: 'Download for Windows' },
  mac: { asset: 'dmg', label: 'Download for macOS' },
  linux: { asset: 'appimage', label: 'Download for Linux' },
};

function detectOS() {
  const ua = navigator.userAgent;
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return 'mobile';
  // iPadOS reports itself as a Mac; touch support gives it away.
  if (/Mac/i.test(ua)) return navigator.maxTouchPoints > 1 ? 'mobile' : 'mac';
  if (/Win/i.test(ua)) return 'windows';
  if (/Linux|X11|CrOS/i.test(ua)) return 'linux';
  return null;
}

async function latestRelease() {
  try {
    const cached = sessionStorage.getItem(CACHE_KEY);
    if (cached) return JSON.parse(cached);
  } catch {
    // Storage blocked: fetch every time.
  }

  const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: { Accept: 'application/vnd.github+json' },
  });
  if (!res.ok) throw new Error(`GitHub API: ${res.status}`);
  const data = await res.json();
  const release = {
    tag: data.tag_name,
    published: data.published_at,
    assets: data.assets.map((a) => ({ name: a.name, url: a.browser_download_url, size: a.size })),
  };

  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(release));
  } catch {
    // Not worth failing over.
  }
  return release;
}

function formatSize(bytes) {
  return `${Math.round(bytes / 1e6)} MB`;
}

function findAsset(release, key) {
  return release.assets.find((a) => ASSETS[key].test(a.name));
}

function applyRelease(release, os) {
  for (const link of document.querySelectorAll('a[data-asset]')) {
    const asset = findAsset(release, link.dataset.asset);
    if (!asset) continue;
    link.href = asset.url;
    // Sizes go beside the file lists, not inside the install guides' buttons.
    if (!link.closest('.files')) continue;
    const size = document.createElement('span');
    size.className = 'size';
    size.textContent = ` · ${formatSize(asset.size)}`;
    link.after(size);
  }

  // The install guides have no OS-picked button; their links carry data-asset.
  const primaryButton = document.getElementById('primary-download');
  const asset = PRIMARY[os] && findAsset(release, PRIMARY[os].asset);
  if (primaryButton && asset) primaryButton.href = asset.url;

  const releaseLine = document.getElementById('release-line');
  if (!releaseLine) return;
  const date = new Date(release.published).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
  releaseLine.textContent = `Version ${release.tag.replace(/^v/, '')} · released ${date}`;
}

function show(id, visible) {
  const el = document.getElementById(id);
  if (el) el.hidden = !visible;
}

const os = detectOS();
const primaryButton = document.getElementById('primary-download');

if (os === 'mobile') {
  show('mobile-note', true);
  show('download-lede', false);
  show('other-platforms', false);
  if (primaryButton) primaryButton.textContent = 'Get it for your computer';
} else if (PRIMARY[os]) {
  if (primaryButton) primaryButton.textContent = PRIMARY[os].label;
  document.querySelector(`.platform[data-os="${os}"]`)?.classList.add('is-current');
}

latestRelease()
  .then((release) => applyRelease(release, os))
  .catch(() => {
    // Leave the links pointing at the releases page.
  });
