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
 * The same word with its verse number removed, or the word unchanged where it
 * cannot be removed.
 *
 * 19 of the 6,236 verses fall into that second case. In those — the sajdah
 * verses, mostly — the whole cluster of word, sajdah mark and number is a
 * single indivisible ligature glyph, and no string operation separates it.
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
