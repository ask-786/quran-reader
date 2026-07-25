/**
 * TypeScript types mirroring the Rust models and the SQLite schema.
 * These are the shapes of all data flowing over the Tauri IPC bridge.
 *
 * Attribution: Quran text provided by Tanzil Project (tanzil.net) — CC BY 3.0
 */

// =============================================================================
// SURAH
// =============================================================================

export type RevelationType = 'Makki' | 'Madani';

export interface Surah {
  id: number; // 1–114, Mushaf order
  name_ar: string; // Arabic name e.g. البقرة
  name_en: string; // English name e.g. The Cow
  transliteration: string; // e.g. Al-Baqarah
  revelation_type: RevelationType;
  verses_count: number;
  order_of_revelation: number; // Chronological order 1–114
  has_bismillah: boolean; // false only for Surah 9
}

// =============================================================================
// AYAH
// =============================================================================

export interface Ayah {
  id: number; // Global sequential id
  surah_id: number;
  ayah_number: number; // Resets per surah
  uthmani_text: string; // Full Uthmani script with diacritics
  simple_text: string; // Simplified Arabic (for search)
  juz: number; // 1–30
  hizb: number; // 1–60
  rub_hizb: number; // 1–240
  manzil: number; // 1–7
  ruku: number; // Sequential ruku number
  page: number; // 1–604 (Madinah Mushaf)
  sajdah: boolean; // true = prostration verse
}

export interface AyahRef {
  id: number;
  surah_id: number;
  ayah_number: number;
  page: number;
  juz: number;
}

// =============================================================================
// TRANSLATION
// =============================================================================

export interface Translation {
  id: number;
  language: string; // BCP-47 e.g. "en", "ml", "ar"
  translator: string;
  title: string;
  version: string;
  is_bundled: boolean;
}

export interface TranslationAyah {
  translation_id: number;
  ayah_id: number;
  text: string;
}

// =============================================================================
// TAFSIR
// =============================================================================

export interface Tafsir {
  id: number;
  language: string;
  author: string;
  title: string;
  version: string;
  is_bundled: boolean;
}

export interface TafsirAyah {
  tafsir_id: number;
  ayah_id: number;
  text: string;
}

// =============================================================================
// BOOKMARK
// =============================================================================

export interface Bookmark {
  id: number;
  ayah_id: number;
  label: string | null;
  created_at: string; // ISO 8601 UTC
}

// =============================================================================
// NOTE
// =============================================================================

export interface Note {
  id: number;
  ayah_id: number;
  content: string;
  created_at: string;
  updated_at: string;
}

// =============================================================================
// SETTINGS
// =============================================================================

export type Theme = 'dark' | 'light' | 'sepia';
export type Font = 'amiri-quran' | 'noto-naskh-arabic';
export type ReaderWidth = 'narrow' | 'normal' | 'wide';

export interface Settings {
  theme: Theme;
  font: Font;
  font_size: number;
  line_height: number;
  reader_width: ReaderWidth;
  last_read_surah_id: number;
  last_read_ayah_id: number;
  preferred_translation_id: number | null;
  show_translation: boolean;
  show_transliteration: boolean;
  show_ayah_numbers: boolean;
  scroll_position: number;
  app_zoom: number;
  reader_zoom: number;
}

// =============================================================================
// SEARCH
// =============================================================================

export interface SearchResult {
  ayah_id: number;
  surah_id: number;
  ayah_number: number;
  uthmani_text: string;
  simple_text: string;
  snippet: string; // FTS5 highlighted snippet
  page: number;
  juz: number;
}

// =============================================================================
// MUSHAF PAGE LAYOUT
// =============================================================================

export type PageLineType = 'surah_header' | 'basmala' | 'text';

export interface PageLineWord {
  position: number;
  ayah_id: number | null;
  word_index: number | null; // 1-based word position within the ayah
  uthmani_text: string; // plain text — search/copy/screen readers
  glyph_v2: string; // QCF v2 glyph string — render with the line's font-family
}

export interface PageLine {
  line_number: number;
  line_type: PageLineType;
  surah_id: number | null; // set for surah_header lines
  first_ayah_id: number | null; // set for text lines
  last_ayah_id: number | null;
  text: string | null; // surah_header: plain Arabic surah name
  words: PageLineWord[];
}

export interface MushafPage {
  page: number; // 1–604
  lines: PageLine[];
}

// =============================================================================
// RICH AYAH VIEW
// =============================================================================

export interface AyahView {
  ayah: Ayah;
  translation: string | null;
  tafsir: string | null;
  is_bookmarked: boolean;
  has_note: boolean;
}

// =============================================================================
// DB STATS
// =============================================================================

export interface DbStats {
  surah_count: number;
  ayah_count: number;
  bookmark_count: number;
  note_count: number;
  schema_version: number;
}
