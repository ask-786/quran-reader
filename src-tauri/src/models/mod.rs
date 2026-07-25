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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TafsirAyah {
    pub tafsir_id: u32,
    pub ayah_id: u32,
    pub text: String,
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
    pub last_read_surah_id: u32,
    pub last_read_ayah_id: u32,
    pub preferred_translation_id: Option<u32>,
    pub show_translation: bool,
    pub show_transliteration: bool,
    pub show_ayah_numbers: bool,
    pub scroll_position: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".to_string(),
            font: "amiri-quran".to_string(),
            font_size: 28,
            line_height: 2.2,
            reader_width: "normal".to_string(),
            last_read_surah_id: 1,
            last_read_ayah_id: 1,
            preferred_translation_id: None,
            show_translation: true,
            show_transliteration: false,
            show_ayah_numbers: true,
            scroll_position: 0,
        }
    }
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
// Line-by-line Madani print layout (QCF v2 glyphs) for a single Mushaf page.
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLineWord {
    pub position: u32,
    pub ayah_id: Option<u32>,
    /// 1-based word position within the Ayah.
    pub word_index: Option<u32>,
    /// Plain Uthmani text — search/copy/screen-reader fallback.
    pub uthmani_text: String,
    /// QCF v2 glyph string. Render with `font-family: 'QCF_P{page:03}'`
    /// (or `'QCF_BSML'` for a basmala line's single word row).
    pub glyph_v2: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AyahView {
    pub ayah: Ayah,
    pub translation: Option<String>,
    pub tafsir: Option<String>,
    pub is_bookmarked: bool,
    pub has_note: bool,
}
