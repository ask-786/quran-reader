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
  slug: string | null;
  /** Set when this edition is a translation of the work rather than the original. */
  translator: string | null;
  name_native: string | null;
  direction: 'ltr' | 'rtl';
  /** Madhhab and creed of the commentary — shown in the picker, since neither
   *  is visible in the text and both decide how it reads whole classes of verse. */
  school: string | null;
  creed: string | null;
}

/**
 * A tafsir edition that can be downloaded, and whether it is already here.
 *
 * Separate from `Tafsir`, which describes an edition the database *has*: this
 * is what the app knows about one before any of it has been fetched, so it
 * carries sizes and a licence rather than an id and a direction.
 */
export interface TafsirPack {
  slug: string;
  title: string;
  author: string;
  language: string;
  license: string;
  /** Size of the download itself. */
  download_bytes: number;
  /** Roughly what it adds to the database once installed. */
  installed_bytes: number;
  installed: boolean;
}

/** Payload of the `tafsir-pack-progress` event, emitted while downloading. */
export interface TafsirPackProgress {
  slug: string;
  received: number;
  total: number;
}

export interface TafsirEntry {
  tafsir_id: number;
  ayah_id: number;
  surah_id: number;
  ayah_number: number;
  text: string;
  /** Verse keys ("2:1" / "2:5") when the edition comments on a run of verses
   *  at once. Both null for per-Ayah editions such as al-Jalalayn. */
  group_start_key: string | null;
  group_end_key: string | null;
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
// RECITATION AUDIO
// =============================================================================

/** A reciter the app can fetch. The catalogue is compiled into the binary and
 *  written to the `reciter` table on every launch. */
export interface Reciter {
  id: number;
  /** The id the CDN knows them by, and the cache directory name. */
  slug: string;
  name_ar: string;
  name_en: string;
  /** Which reading. Almost always Ḥafṣ ʿan ʿĀṣim, which is why the exception
   *  needs to say so. */
  riwaya: string;
  style: 'murattal' | 'mujawwad';
  source_url: string | null;
  license: string | null;
}

/** What one reciter's cached audio costs on disk. */
export interface ReciterUsage {
  reciter_id: number;
  slug: string;
  name_en: string;
  files: number;
  bytes: number;
}

/** Payload of `audio-download-progress`, emitted while fetching a range. */
export interface AudioProgress {
  slug: string;
  done: number;
  total: number;
  ayah_id: number;
  /** Verses this run could not fetch. It carries on past them. */
  failed: number;
}

// =============================================================================
// SETTINGS
// =============================================================================

export type Theme = 'dark' | 'light' | 'sepia';
/** Which surface a tafsir trigger opens — see docs/tafsir-popover-plan.md. */
export type TafsirView = 'popover' | 'panel';
export type Font = 'amiri-quran' | 'noto-naskh-arabic';
export type ReaderWidth = 'narrow' | 'normal' | 'wide';

/**
 * What Mushaf view does with the parts of a printed page that fall outside the
 * range you opened — the tail of the previous Surah above, the head of the next
 * one below. `all` prints the page as it stands, `dim` keeps those lines as
 * faint context, `trim` drops them so the range is the only thing on the page.
 */
export type RangeFocus = 'all' | 'dim' | 'trim';

/** What happens when a verse finishes. `range` repeats the whole queue. */
export type RepeatMode = 'off' | 'ayah' | 'range';

/** The two rates the CDN serves. 32, 48 and 192 kbps are 403 there. */
export type AudioBitrate = 64 | 128;

export interface Settings {
  theme: Theme;
  font: Font;
  font_size: number;
  line_height: number;
  reader_width: ReaderWidth;
  preferred_translation_id: number | null;
  show_translation: boolean;
  /** Null until an edition is chosen; the app falls back to the first bundled one. */
  tafsir_id: number | null;
  /** The side panel's state. Popover openness is transient and not persisted. */
  show_tafsir: boolean;
  tafsir_panel_width: number;
  tafsir_view: TafsirView;
  /** Whether clicking a verse opens its commentary. Off by default — a stray
   *  click in the reader should not become an interruption. */
  tafsir_click: boolean;
  show_transliteration: boolean;
  show_ayah_numbers: boolean;
  range_focus: RangeFocus;
  app_zoom: number;
  reader_zoom_normal: number;
  reader_zoom_focus: number;
  /** Null until a reciter is chosen. No reciter means the app fetches nothing. */
  reciter_id: number | null;
  audio_bitrate: AudioBitrate;
  audio_repeat_mode: RepeatMode;
  audio_repeat_count: number;
  /** Silence between repetitions, for saying the verse back. */
  audio_repeat_pause_ms: number;
  audio_playback_rate: number;
  /** Whether the reader scrolls to the verse being recited. */
  audio_follow: boolean;
  audio_volume: number;
  /** The network switch. Off until the reader approves the first fetch; off
   *  again is "cached only" — downloaded verses still play, nothing goes out. */
  audio_downloads_allowed: boolean;
}

// =============================================================================
// READING POSITION / HISTORY
// =============================================================================

/**
 * Which kind of range was open. The three navigable divisions plus the Mushaf
 * page route — one per reader route, which is what lets a position be restored
 * by navigating back to where it was read.
 */
export type ReadingScope = 'surah' | 'juz' | 'hizb' | 'page';

/** Where the reader left off inside one range. */
export interface ReadingPosition {
  scope: ReadingScope;
  scope_id: number;
  ayah: AyahRef; // resolved from the stored ayah id, so it can be labelled directly
  updated_at: string; // ISO 8601 UTC
}

/** One sitting: where a stretch of reading began and how far it reached. */
export interface ReadingSession {
  id: number;
  scope: ReadingScope;
  scope_id: number;
  start: AyahRef;
  end: AyahRef;
  started_at: string;
  updated_at: string;
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
  glyph_v4: string | null; // QCF v4 glyph string — render with the page's font-map family.
  // Null for the handful of rows the v4 import couldn't attach a glyph to.
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
// READER — PAGE-GLYPH WORDS
// =============================================================================

export interface GlyphSpan {
  uthmani_text: string;
  glyph_v4: string | null;
}

/**
 * A single word rendered from Mushaf page-glyph data (see PageLineWord)
 * rather than live-shaped Unicode text, tagged with the font-family of the
 * page it came from — an Ayah split across a page boundary has words in two
 * different QCF fonts.
 */
export interface AyahGlyphWord extends GlyphSpan {
  fontFamily: string | null;
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
