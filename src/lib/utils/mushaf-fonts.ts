/**
 * Lazily loads the QCF v4 Mushaf fonts (47 font groups covering all 604
 * pages, ~13 pages per group, plus one surah-header/basmala font) via the
 * CSS Font Loading API instead of declaring every @font-face rule up front.
 *
 * Fonts are loaded one group at a time and kept in an LRU, rather than as an
 * all-or-nothing active set. See docs/qcf-v4-font-migration-plan.md for the
 * v2 -> v4 migration this replaced: unlike v2 (one font per page, so the
 * family name was derivable from the page number alone), a v4 page's font
 * family has to be looked up in font-map.json — several consecutive pages
 * share one font group — so every export here is keyed by page but caches
 * and fetches by *family*, and the map has to be loaded before the first
 * lookup (see `loadFontMap`).
 *
 * The v2 loader below is kept, unused, so a rollback to v2 rendering is a
 * font-loader import swap in the render components rather than a rewrite —
 * see the migration plan's Rollback section. `static/fonts/mushaf/` (the v2
 * font files) and `page_line_word.glyph_v2` are both still shipped.
 */

/**
 * The Basmala glyph lives in `QCF4_Hafs_01` — *not* in `QCF4_QBSML`, despite
 * what that font's name suggests. QBSML holds the Surah-title banner glyphs,
 * which this app never renders (SurahHeader draws the name live from
 * `surah.name_ar` in Scheherazade New); QBSML's copies of the Basmala
 * codepoints are zero-contour blanks, so rendering the Basmala with it
 * silently produces an empty line.
 *
 * Every v4 page's `bismillah` entry names `QCF4_Hafs_01` regardless of which
 * font group the page itself belongs to, so this is a fixed family rather
 * than a per-page lookup. It is also pages 1–13's own group font, which is
 * why it shares the family registry below instead of holding its own
 * FontFace — two FontFaces for one family would double-fetch the same file.
 */
const BASMALA_FAMILY = 'QCF4_Hafs_01';

/**
 * Basmala lines use a single whole-phrase glyph. Loaded once and kept for the
 * app's lifetime (never trimmed — see `trimPageFonts`), since it is reused on
 * every Surah-opening page.
 */
export function loadBasmalaFont(): Promise<string> {
  return ensureFamily(BASMALA_FAMILY);
}

/**
 * page -> font-group family, e.g. "1" -> "QCF4_Hafs_01". Populated once by
 * `loadFontMap`; every export below that resolves a page to a family assumes
 * it has already been awaited (every caller in PageView/ReaderView loads it
 * alongside `loadPages`/`loadBasmalaFont`, before rendering anything that
 * needs a family name).
 */
let fontMap: Record<string, string> = {};
let fontMapPromise: Promise<void> | null = null;

export function loadFontMap(): Promise<void> {
  if (!fontMapPromise) {
    fontMapPromise = fetch('/fonts/mushaf-v4/font-map.json')
      .then((r) => r.json())
      .then((map: Record<string, string>) => {
        fontMap = map;
      });
  }
  return fontMapPromise;
}

/** How many font *groups* to keep registered — see MAX_FONTS' v2 history
 * below for why this is so much lower than 64 despite covering more pages:
 * each v4 group is a bigger file (~761KB mean vs ~155KB), and covers ~13
 * pages instead of 1, so 8 groups already spans roughly 100 pages — more
 * render-window coverage than 64 v2 fonts gave, for a fraction of the
 * resident memory (~6MB vs the ~48MB 64 v4 groups would cost). */
const MAX_FONTS = 8;

/** Insertion order is LRU order. Keyed by family, not page: several pages
 * share one family, and page-keying would fetch the same file repeatedly. */
const loaded = new Map<string, FontFace>();
const inFlight = new Map<string, Promise<string>>();

export function familyForPage(page: number): string | undefined {
  return fontMap[String(page)];
}

function urlForFamily(family: string): string {
  return `/fonts/mushaf-v4/${family}_W.woff2`;
}

/** True if this page's font group is registered and its glyphs will render now. */
export function isPageFontReady(page: number): boolean {
  const family = familyForPage(page);
  return family !== undefined && loaded.has(family);
}

/**
 * Load and register the font group covering `page`, returning its family
 * name. Concurrent callers for the same (or a co-grouped) page share a
 * single load.
 */
export function ensurePageFont(page: number): Promise<string> {
  const family = familyForPage(page);
  if (!family) return Promise.reject(new Error(`No QCF v4 font family for page ${page}`));
  return ensureFamily(family);
}

/**
 * Load and register one font group by family name. Concurrent callers for the
 * same family — including a page load and the Basmala load racing each other
 * over `QCF4_Hafs_01` — share a single fetch.
 */
function ensureFamily(family: string): Promise<string> {
  const hit = loaded.get(family);
  if (hit) {
    // Re-insert to mark as most recently used.
    loaded.delete(family);
    loaded.set(family, hit);
    return Promise.resolve(family);
  }

  let pending = inFlight.get(family);
  if (!pending) {
    pending = (async () => {
      const face = new FontFace(family, `url(${urlForFamily(family)})`);
      await face.load();
      document.fonts.add(face);
      loaded.set(family, face);
      return family;
    })().finally(() => inFlight.delete(family));
    inFlight.set(family, pending);
  }
  return pending;
}

/**
 * Drop registered font groups down to the LRU cap, never touching a group
 * needed by any page in `retainedPages`. Callers pass the set of pages whose
 * glyphs are currently rendered; any group none of them need is fair game
 * once the cap is exceeded.
 */
export function trimPageFonts(retainedPages: Iterable<number>): void {
  if (loaded.size <= MAX_FONTS) return;

  // The Basmala family is pinned, never trimmed. It is also pages 1–13's own
  // group font, so without this it would be evicted as soon as the reader
  // scrolled far enough from the start of the Mushaf — and every Surah banner
  // from then on would render a blank Basmala, anywhere in the Mushaf.
  const keepFamilies = new Set<string>([BASMALA_FAMILY]);
  for (const page of retainedPages) {
    const family = familyForPage(page);
    if (family) keepFamilies.add(family);
  }

  for (const [family, face] of loaded) {
    if (loaded.size <= MAX_FONTS) break;
    if (keepFamilies.has(family)) continue;
    document.fonts.delete(face);
    loaded.delete(family);
  }
}

// =============================================================================
// QCF v2 (legacy) — unused by the render components, kept so rollback is an
// import swap rather than a rewrite. See the module doc comment above.
// =============================================================================

const BASMALA_FAMILY_V2 = 'QCF_BSML';
let basmalaFontPromiseV2: Promise<string> | null = null;

export function loadBasmalaFontV2(): Promise<string> {
  if (!basmalaFontPromiseV2) {
    basmalaFontPromiseV2 = (async () => {
      const face = new FontFace(BASMALA_FAMILY_V2, `url(/fonts/mushaf/${BASMALA_FAMILY_V2}.woff2)`);
      await face.load();
      document.fonts.add(face);
      return BASMALA_FAMILY_V2;
    })();
  }
  return basmalaFontPromiseV2;
}

const MAX_FONTS_V2 = 64;
const loadedV2 = new Map<number, FontFace>();
const inFlightV2 = new Map<number, Promise<string>>();

export function familyForPageV2(page: number): string {
  return `QCF_P${String(page).padStart(3, '0')}`;
}

function urlForPageV2(page: number): string {
  return `/fonts/mushaf/QCF_P${String(page).padStart(3, '0')}.woff2`;
}

export function isPageFontReadyV2(page: number): boolean {
  return loadedV2.has(page);
}

export function ensurePageFontV2(page: number): Promise<string> {
  const family = familyForPageV2(page);

  const hit = loadedV2.get(page);
  if (hit) {
    loadedV2.delete(page);
    loadedV2.set(page, hit);
    return Promise.resolve(family);
  }

  let pending = inFlightV2.get(page);
  if (!pending) {
    pending = (async () => {
      const face = new FontFace(family, `url(${urlForPageV2(page)})`);
      await face.load();
      document.fonts.add(face);
      loadedV2.set(page, face);
      return family;
    })().finally(() => inFlightV2.delete(page));
    inFlightV2.set(page, pending);
  }
  return pending;
}

export function trimPageFontsV2(retained: Iterable<number>): void {
  const keep = new Set(retained);
  if (loadedV2.size <= MAX_FONTS_V2) return;

  for (const [page, face] of loadedV2) {
    if (loadedV2.size <= MAX_FONTS_V2) break;
    if (keep.has(page)) continue;
    document.fonts.delete(face);
    loadedV2.delete(page);
  }
}
