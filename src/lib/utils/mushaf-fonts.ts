/**
 * Lazily loads the QCF v4 Mushaf fonts (47 font groups covering all 604
 * pages, ~13 pages per group) via the CSS Font Loading API instead of
 * declaring every @font-face rule up front.
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
 * v2 is fully gone as of the strip commit — its 604 font files, its loader
 * here, and `page_line_word.glyph_v2`. Rollback is a revert of that commit,
 * not a runtime fallback; shipping both sets was a 131 MB bundle, larger than
 * the v2-only one the migration set out to shrink.
 */

/**
 * page -> font-group family, e.g. "1" -> "QCF4_Hafs_01". Populated once by
 * `loadFontMap`; every export below that resolves a page to a family assumes
 * it has already been awaited (every caller in PageView/ReaderView loads it
 * alongside `loadPages`, before rendering anything that needs a family name).
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
 * same family (co-grouped pages) share a single fetch.
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

/** Every page from `start` to `end`, inclusive. */
export function pageSpan(start: number, end: number): number[] {
  return Array.from({ length: Math.max(0, end - start + 1) }, (_, i) => start + i);
}

/**
 * Load every font group `pages` need that isn't registered yet, and register
 * them together.
 *
 * Registering a group restyles the whole document, and how much that costs
 * depends on how much text is rendered. Left to `ensurePageFont` as reading
 * reaches each group, it lands mid-range with a dozen pages of glyphs in the
 * DOM: a quarter-second freeze in the middle of auto-scroll on Al-Baqara. Run
 * once a range has first painted instead, only the few pages around the
 * landing spot are rendered, and the groups are fetched in parallel and added
 * in a single task, so the document is restyled once for all of them.
 *
 * A whole range is at most four groups (Al-Baqara, the longest Surah, and any
 * Juz), comfortably under MAX_FONTS. Callers retain the whole range when
 * trimming so none of them are evicted again before they're used.
 */
export function preloadPageFonts(pages: Iterable<number>): Promise<void> {
  const families = [
    ...new Set(
      [...pages]
        .map(familyForPage)
        .filter((f): f is string => f !== undefined && !loaded.has(f) && !inFlight.has(f)),
    ),
  ];
  if (families.length === 0) return Promise.resolve();

  const loads = families.map((family) =>
    new FontFace(family, `url(${urlForFamily(family)})`).load(),
  );
  const registered = Promise.allSettled(loads).then((results) => {
    results.forEach((result, i) => {
      if (result.status !== 'fulfilled') return;
      document.fonts.add(result.value);
      loaded.set(families[i], result.value);
    });
  });

  // A page reaching the render window mid-batch waits on the batch rather
  // than fetching its group a second time.
  families.forEach((family, i) => {
    const pending = registered
      .then(() => loads[i])
      .then(() => family)
      .finally(() => inFlight.delete(family));
    // Unwatched unless a page asks for it; a failure here is reported to
    // whoever does.
    pending.catch(() => {});
    inFlight.set(family, pending);
  });

  return registered;
}

/**
 * Drop registered font groups down to the LRU cap, never touching a group
 * needed by any page in `retainedPages`. Callers pass the pages of the open
 * range — every group `preloadPageFonts` registered for it has to survive
 * until it's read — and any group none of them need is fair game once the
 * cap is exceeded.
 */
export function trimPageFonts(retainedPages: Iterable<number>): void {
  if (loaded.size <= MAX_FONTS) return;

  // The Basmala is not a font concern — SurahHeader draws it as SVG (see
  // scripts/generate-basmala-paths.py) — so no family needs pinning here.
  const keepFamilies = new Set<string>();
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
