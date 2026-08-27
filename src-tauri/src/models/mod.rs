//! Rust structs that mirror the database tables.
//! All structs derive `serde::Serialize` so they can be returned
//! from Tauri commands to the frontend.

use serde::{Deserialize, Serialize};

// =============================================================================
// SURAH
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surah {
    pub id: u32,
    pub name_ar: String,
    pub name_en: String,
    pub transliteration: String,
    pub revelation_type: RevelationType,
    pub verses_count: u32,
    pub order_of_revelation: u32,
    pub has_bismillah: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RevelationType {
    Makki,
    Madani,
}

impl std::str::FromStr for RevelationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Makki" => Ok(RevelationType::Makki),
            "Madani" => Ok(RevelationType::Madani),
            other => Err(format!("Unknown revelation type: {}", other)),
        }
    }
}

impl std::fmt::Display for RevelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevelationType::Makki => write!(f, "Makki"),
            RevelationType::Madani => write!(f, "Madani"),
        }
    }
}

// =============================================================================
// AYAH
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ayah {
    pub id: u32,
    pub surah_id: u32,
    pub ayah_number: u32,
    pub uthmani_text: String,
    pub simple_text: String,
    pub juz: u32,
    pub hizb: u32,
    pub rub_hizb: u32,
    pub manzil: u32,
    pub ruku: u32,
    pub page: u32,
    pub sajdah: bool,
}

/// Lightweight view used in list/navigation contexts (omits full text).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AyahRef {
    pub id: u32,
    pub surah_id: u32,
    pub ayah_number: u32,
    pub page: u32,
    pub juz: u32,
}

// =============================================================================
// TRANSLATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub id: u32,
    pub language: String,
    pub translator: String,
    pub title: String,
    pub version: String,
    pub is_bundled: bool,
}

#[allow(dead_code)] // wired up by Phase 10 — Translations (PLAN.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationAyah {
    pub translation_id: u32,
    pub ayah_id: u32,
    pub text: String,
}

// =============================================================================
// TAFSIR
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tafsir {
    pub id: u32,
    pub language: String,
    pub author: String,
    pub title: String,
    pub version: String,
    pub is_bundled: bool,
    /// Stable source id, e.g. "tafsir-al-jalalayn".
    pub slug: Option<String>,
    /// Set when the edition is a translation of the work rather than the original.
    pub translator: Option<String>,
    pub name_native: Option<String>,
    /// "ltr" | "rtl" — the edition's own language, not the app's.
    pub direction: String,
    /// Madhhab and creed of the commentary. Surfaced in the picker: which
    /// school a tafsir belongs to decides how it reads the legal verses, and
    /// its creed decides how it reads the attribute verses, neither of which
    /// is visible in the text itself.
    pub school: Option<String>,
    pub creed: Option<String>,
}

/// A single commentary entry, joined with the Ayah it belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TafsirEntry {
    pub tafsir_id: u32,
    pub ayah_id: u32,
    pub surah_id: u32,
    pub ayah_number: u32,
    pub text: String,
    /// Verse keys ("2:1" / "2:5") when this edition comments on a run of
    /// verses at once, so the panel can label the run instead of implying the
    /// text belongs to this Ayah alone. Both null for per-Ayah editions.
    pub group_start_key: Option<String>,
    pub group_end_key: Option<String>,
}

// =============================================================================
// BOOKMARK
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: u32,
    pub ayah_id: u32,
    pub label: Option<String>,
    pub created_at: String,
}

// =============================================================================
// NOTE
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: u32,
    pub ayah_id: u32,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

// =============================================================================
// SETTINGS
// =============================================================================

/// Typed view of all user settings loaded from the key-value store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub font: String,
    pub font_size: u32,
    pub line_height: f32,
    pub reader_width: String,
    pub preferred_translation_id: Option<u32>,
    pub show_translation: bool,
    /// None until a tafsir is chosen; the app then falls back to the first
    /// bundled edition.
    pub tafsir_id: Option<u32>,
    /// The side panel's open state. The popover is transient and is not
    /// persisted at all — see docs/tafsir-popover-plan.md.
    pub show_tafsir: bool,
    pub tafsir_panel_width: u32,
    /// "popover" | "panel" — which surface a tafsir trigger opens.
    pub tafsir_view: String,
    /// Whether clicking a verse opens its commentary. Off by default: with the
    /// popover on every click, a stray click in the reader is an interruption
    /// rather than an answer. The per-Ayah button and `t` open it either way.
    pub tafsir_click: bool,
    pub show_transliteration: bool,
    pub show_ayah_numbers: bool,
    /// "all" | "dim" | "trim" — what the Mushaf page view does with the parts
    /// of a printed page outside the opened range. A page is shared between
    /// Surahs, and opening one Surah should not hand you the tail of the one
    /// before it.
    pub range_focus: String,
    pub app_zoom: f32,
    pub reader_zoom_normal: f32,
    pub reader_zoom_focus: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".to_string(),
            font: "amiri-quran".to_string(),
            font_size: 28,
            line_height: 2.2,
            reader_width: "normal".to_string(),
            preferred_translation_id: None,
            show_translation: true,
            tafsir_id: None,
            show_tafsir: false,
            tafsir_panel_width: 420,
            tafsir_view: "popover".to_string(),
            tafsir_click: false,
            show_transliteration: false,
            show_ayah_numbers: true,
            range_focus: "trim".to_string(),
            app_zoom: 1.0,
            reader_zoom_normal: 1.0,
            reader_zoom_focus: 1.0,
        }
    }
}

// =============================================================================
// READING POSITION / HISTORY
// =============================================================================

/// Which kind of range the reader had open. The three navigable divisions plus
/// the Mushaf page route, matching `reading_position.scope` in the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadingScope {
    Surah,
    Juz,
    Hizb,
    Page,
}

impl ReadingScope {
    pub fn as_str(self) -> &'static str {
        match self {
            ReadingScope::Surah => "surah",
            ReadingScope::Juz => "juz",
            ReadingScope::Hizb => "hizb",
            ReadingScope::Page => "page",
        }
    }
}

impl std::str::FromStr for ReadingScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "surah" => Ok(ReadingScope::Surah),
            "juz" => Ok(ReadingScope::Juz),
            "hizb" => Ok(ReadingScope::Hizb),
            "page" => Ok(ReadingScope::Page),
            other => Err(format!("Unknown reading scope: {}", other)),
        }
    }
}

/// Where the reader last was inside one range. `ayah` carries the Surah, Ayah
/// number, page and Juz the stored id resolves to, so a caller can label the
/// position without a second lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingPosition {
    pub scope: ReadingScope,
    pub scope_id: u32,
    pub ayah: AyahRef,
    pub updated_at: String,
}

/// One sitting: where a stretch of reading started and how far it reached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSession {
    pub id: u32,
    pub scope: ReadingScope,
    pub scope_id: u32,
    pub start: AyahRef,
    pub end: AyahRef,
    pub started_at: String,
    pub updated_at: String,
}

// =============================================================================
// SEARCH
// =============================================================================

/// A single search hit returned by the FTS queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub ayah_id: u32,
    pub surah_id: u32,
    pub ayah_number: u32,
    pub uthmani_text: String,
    pub simple_text: String,
    /// Highlighted snippet (FTS5 snippet function output)
    pub snippet: String,
    pub page: u32,
    pub juz: u32,
}

// =============================================================================
// MUSHAF PAGE LAYOUT
// Line-by-line Madani print layout (QCF v4 glyphs) for a single Mushaf page.
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLineWord {
    pub position: u32,
    pub ayah_id: Option<u32>,
    /// 1-based word position within the Ayah.
    pub word_index: Option<u32>,
    /// Plain Uthmani text — search/copy/screen-reader fallback.
    pub uthmani_text: String,
    /// QCF v4 glyph string. Render with the font-map.json family for this
    /// row's page. A basmala line's single word row instead uses
    /// `'QCF4_Hafs_01'`, which holds the basmala glyph for every page —
    /// *not* `'QCF4_QBSML'`, whose copies of those codepoints are blank.
    /// Null for the handful of rows the v4 import couldn't attach a glyph to
    /// (see `mushaf::write_glyph_v4`'s doc comment in the importer).
    pub glyph_v4: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLine {
    pub line_number: u32,
    /// "surah_header" | "basmala" | "text"
    pub line_type: String,
    /// Set for surah_header lines.
    pub surah_id: Option<u32>,
    /// Set for text lines (an Ayah can span multiple consecutive lines).
    pub first_ayah_id: Option<u32>,
    pub last_ayah_id: Option<u32>,
    /// surah_header: plain Arabic Surah name, rendered with the QCF header font.
    pub text: Option<String>,
    pub words: Vec<PageLineWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MushafPage {
    pub page: u32,
    pub lines: Vec<PageLine>,
}

// =============================================================================
// RICH AYAH (joined view for reader rendering)
// =============================================================================

/// Full data needed to render a single Ayah in the reader, including optional
/// translation text for the active translation.
#[allow(dead_code)] // wired up alongside Phase 10/11 — Translations/Tafsir (PLAN.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AyahView {
    pub ayah: Ayah,
    pub translation: Option<String>,
    pub tafsir: Option<String>,
    pub is_bookmarked: bool,
    pub has_note: bool,
}
