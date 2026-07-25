/**
 * Lazily loads the per-page QCF v2 Mushaf fonts (one font file per Mushaf
 * page — 604 files, ~48MB total) via the CSS Font Loading API instead of
 * declaring 604 @font-face rules up front. Fonts for the currently active
 * set of pages (e.g. every Mushaf page spanned by the Surah on screen) are
 * kept registered for as long as they're active; loading a new active set
 * evicts fonts left over from the previous one, so long reading sessions
 * don't grow memory unbounded.
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

const loaded = new Map<string, FontFace>(); // family name -> FontFace

function familyForPage(page: number): string {
  return `QCF_P${String(page).padStart(3, '0')}`;
}

function urlForPage(page: number): string {
  return `/fonts/mushaf/QCF_P${String(page).padStart(3, '0')}.woff2`;
}

/**
 * Ensure fonts for exactly this set of pages are loaded and registered,
 * evicting any fonts left over from a previously active set that this one
 * doesn't need. Returns page -> font-family. Callers that render several
 * pages at once (e.g. a Surah's full continuous-scroll page range) must call
 * this with the whole set together — loading pages one at a time via
 * multiple calls would have each call evict the pages loaded by the last.
 */
export async function loadPageFonts(pages: number[]): Promise<Map<number, string>> {
  const activeFamilies = new Set(pages.map(familyForPage));

  const result = new Map<number, string>();
  await Promise.all(
    pages.map(async (page) => {
      const family = familyForPage(page);
      if (!loaded.has(family)) {
        const face = new FontFace(family, `url(${urlForPage(page)})`);
        await face.load();
        document.fonts.add(face);
        loaded.set(family, face);
      }
      result.set(page, family);
    }),
  );

  for (const [family, face] of loaded) {
    if (!activeFamilies.has(family)) {
      document.fonts.delete(face);
      loaded.delete(family);
    }
  }

  return result;
}
