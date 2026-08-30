/**
 * Taking the end-of-verse marker off a Mushaf glyph word.
 *
 * The marker is not a word of its own in the page data: it is fused into the
 * verse's last word as a second glyph after a space —
 * `ٱلرَّحِيمِ ١` / `U+FEC3 U+0020 U+FEC4` — so hiding it means splitting that
 * string rather than dropping an element.
 *
 * The digit test, and not the presence of a space, is what identifies such a
 * word. The mirror case is a rubʿ ornament (۞), which is fused onto a verse's
 * *first* word the other way round — leading token, not trailing — and 199 of
 * those exist. Every one of the 6,236 verse-final words ends in an Arabic-Indic
 * digit, and nothing else in the page data does.
 *
 * Used by the reader's scrolling view and by the typography preview in
 * Settings, so that the preview shows the same text the reader will.
 */
import type { GlyphSpan } from '$lib/types/database';

/** Arabic-Indic digits, U+0660–U+0669. */
const ENDS_IN_MARKER = /[٠-٩]$/;
const MARKER_TAIL = /\s*[٠-٩]+$/;

/** Whether this word carries a verse number — i.e. whether it is a verse's last. */
export function hasVerseMarker(word: GlyphSpan): boolean {
  return ENDS_IN_MARKER.test(word.uthmani_text);
}

/**
 * The same word with its verse number removed.
 *
 * The marker is the glyph after the last space, so the cut goes there. A
 * sajdah verse's last word carries three glyphs — word, ۩, marker — and the
 * same cut keeps the ۩ and drops only the number, which is what the printed
 * page shows.
 *
 * This used to bail out on 19 verses whose glyph string held no space to cut
 * at, which read as an indivisible ligature. It wasn't: the layout import was
 * dropping those verses' marker glyph outright (see `importer/src/mushaf.rs`).
 * All 6,236 verse-final words now carry a separable marker.
 */
export function withoutVerseMarker<T extends GlyphSpan>(word: T): T {
  const cut = word.glyph_v4?.lastIndexOf(' ') ?? -1;
  if (cut <= 0) return word;
  return {
    ...word,
    glyph_v4: word.glyph_v4!.slice(0, cut),
    uthmani_text: word.uthmani_text.replace(MARKER_TAIL, ''),
  };
}
