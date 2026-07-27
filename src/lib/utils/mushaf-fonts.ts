/**
 * Lazily loads the per-page QCF v2 Mushaf fonts (one font file per Mushaf
 * page — 604 files, ~95MB total) via the CSS Font Loading API instead of
 * declaring 604 @font-face rules up front.
 *
 * Fonts are loaded one page at a time and kept in an LRU, rather than as an
 * all-or-nothing active set. Loading a whole Surah's range up front was the
 * reader's dominant startup cost — Al-Baqara spans 48 pages, which is 5.9MB of
 * woff2 across 48 fetch-decompress-register cycles, all behind one
 * `Promise.all` barrier before a single word could be painted.
 *
 * Eviction is deliberately conservative: a font is only ever dropped when it
 * is outside the caller's retained set, because a page whose glyphs are still
 * in the DOM renders as tofu the moment its font leaves `document.fonts`.
 */

const BASMALA_FAMILY = 'QCF_BSML';
let basmalaFontPromise: Promise<string> | null = null;

/**
 * Basmala lines use a dedicated glyph font, distinct from every page's own
 * font (its glyph codepoints aren't present in the per-page fonts at all).
 * Loaded once and kept for the app's lifetime — small and reused on every
 * Surah-opening page.
 */
export function loadBasmalaFont(): Promise<string> {
  if (!basmalaFontPromise) {
    basmalaFontPromise = (async () => {
      const face = new FontFace(BASMALA_FAMILY, `url(/fonts/mushaf/${BASMALA_FAMILY}.woff2)`);
      await face.load();
      document.fonts.add(face);
      return BASMALA_FAMILY;
    })();
  }
  return basmalaFontPromise;
}

/**
 * How many page fonts to keep registered. A render window is a handful of
 * pages, so this leaves a lot of slack for scrolling back and forth without
 * refetching, while capping resident font memory at roughly 8MB.
 */
const MAX_FONTS = 64;

/** Insertion order is LRU order. */
const loaded = new Map<number, FontFace>();
const inFlight = new Map<number, Promise<string>>();

export function familyForPage(page: number): string {
  return `QCF_P${String(page).padStart(3, '0')}`;
}

function urlForPage(page: number): string {
  return `/fonts/mushaf/QCF_P${String(page).padStart(3, '0')}.woff2`;
}

/** True if this page's font is registered and its glyphs will render now. */
export function isPageFontReady(page: number): boolean {
  return loaded.has(page);
}

/**
 * Load and register one page's font, returning its family name. Concurrent
 * callers for the same page share a single load.
 */
export function ensurePageFont(page: number): Promise<string> {
  const family = familyForPage(page);

  const hit = loaded.get(page);
  if (hit) {
    // Re-insert to mark as most recently used.
    loaded.delete(page);
    loaded.set(page, hit);
    return Promise.resolve(family);
  }

  let pending = inFlight.get(page);
  if (!pending) {
    pending = (async () => {
      const face = new FontFace(family, `url(${urlForPage(page)})`);
      await face.load();
      document.fonts.add(face);
      loaded.set(page, face);
      return family;
    })().finally(() => inFlight.delete(page));
    inFlight.set(page, pending);
  }
  return pending;
}

/**
 * Drop registered fonts down to the LRU cap, never touching a page in
 * `retained`. Callers pass the set of pages whose glyphs are currently
 * rendered; anything else is fair game once the cap is exceeded.
 */
export function trimPageFonts(retained: Iterable<number>): void {
  const keep = new Set(retained);
  if (loaded.size <= MAX_FONTS) return;

  for (const [page, face] of loaded) {
    if (loaded.size <= MAX_FONTS) break;
    if (keep.has(page)) continue;
    document.fonts.delete(face);
    loaded.delete(page);
  }
}
