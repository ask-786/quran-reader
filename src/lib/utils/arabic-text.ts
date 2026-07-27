/* Escapes rather than literals throughout: these are combining marks, so a
   literal in the source is invisible and easy to mangle in a later edit. */

/** Harakat (fathatan…sukun, plus the hamza/maddah marks that sit with them),
    the dagger alef, and the Quranic annotation signs — U+06D6–U+06ED is where
    the maddah over يسٓ / صٓ / قٓ and the small high sukun in عِمۡرَان live. */
const TASHKEEL = /[\u0617-\u061A\u064B-\u0655\u0670\u06D6-\u06ED]/g;

/** Alef + a standalone maddah is just آ; fold it into the single letter before
    stripping, so آل عمران doesn't come out as ال عمران. */
const ALEF_PLUS_MADDAH = /\u0627\u0653/g;

/** Alef wasla reads as a plain alef once unvocalised. */
const ALEF_WASLA = /\u0671/g;

/** Kashida — in Surah names it is only ever there to carry a harakah (المَائـِدَة). */
const TATWEEL = /\u0640/g;

/**
 * Strip vowel marks from Arabic text, leaving the bare letters — e.g.
 * `سُورَةُ ٱلْفَاتِحَةِ` → `سورة الفاتحة`. Used for Surah names in UI chrome,
 * where the fully vocalised Mushaf spelling is noise, and for matching them
 * against unvocalised search input. Never apply it to Ayah text.
 */
export function stripTashkeel(text: string): string {
  return text
    .replace(ALEF_PLUS_MADDAH, '\u0622')
    .replace(ALEF_WASLA, '\u0627')
    .replace(TASHKEEL, '')
    .replace(TATWEEL, '');
}
