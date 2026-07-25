/**
 * Lazily loads the per-page QCF v2 Mushaf fonts (one font file per Mushaf
 * page — 604 files, ~48MB total) via the CSS Font Loading API instead of
 * declaring 604 @font-face rules up front. Keeps a small LRU of loaded pages
 * so long reading sessions don't grow memory unbounded.
 */

const MAX_LOADED = 5; // current page + a couple of neighbours in either direction

const loaded = new Map<string, FontFace>(); // family name -> FontFace, oldest-first

function familyForPage(page: number): string {
  return `QCF_P${String(page).padStart(3, '0')}`;
}

function urlForPage(page: number): string {
  return `/fonts/mushaf/QCF_P${String(page).padStart(3, '0')}.woff2`;
}

/** Ensure the given page's font is loaded and registered; returns its font-family name. */
export async function loadPageFont(page: number): Promise<string> {
  const family = familyForPage(page);

  const existing = loaded.get(family);
  if (existing) {
    // Bump recency (delete + re-insert moves it to the end of Map's iteration order).
    loaded.delete(family);
    loaded.set(family, existing);
    return family;
  }

  const face = new FontFace(family, `url(${urlForPage(page)})`);
  await face.load();
  document.fonts.add(face);
  loaded.set(family, face);
  evictOldest();

  return family;
}

function evictOldest() {
  while (loaded.size > MAX_LOADED) {
    const oldestKey = loaded.keys().next().value;
    if (oldestKey === undefined) break;
    const face = loaded.get(oldestKey);
    if (face) document.fonts.delete(face);
    loaded.delete(oldestKey);
  }
}
