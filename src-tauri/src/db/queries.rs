//! Database query functions.
//! Each function takes a `&Connection` and returns a typed `DbResult`.
//! No business logic here — pure data access.

use rusqlite::{Connection, params};
use crate::db::error::{DbError, DbResult};
use crate::models::*;

// =============================================================================
// SURAH QUERIES
// =============================================================================

/// Return all 114 Surahs ordered by Mushaf order (id ASC).
pub fn get_all_surahs(conn: &Connection) -> DbResult<Vec<Surah>> {
    let mut stmt = conn.prepare(
        "SELECT id, name_ar, name_en, transliteration, revelation_type,
                verses_count, order_of_revelation, has_bismillah
         FROM surah ORDER BY id ASC"
    )?;

    let surahs = stmt.query_map([], |row| {
        Ok((
            row.get::<_, u32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, u32>(5)?,
            row.get::<_, u32>(6)?,
            row.get::<_, bool>(7)?,
        ))
    })?
    .map(|r| {
        let (id, name_ar, name_en, transliteration, revelation_type_str,
             verses_count, order_of_revelation, has_bismillah) = r?;
        let revelation_type = revelation_type_str.parse::<RevelationType>()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
        Ok(Surah {
            id,
            name_ar,
            name_en,
            transliteration,
            revelation_type,
            verses_count,
            order_of_revelation,
            has_bismillah,
        })
    })
    .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(surahs)
}

/// Return a single Surah by id. Returns `DbError::NotFound` if missing.
pub fn get_surah(conn: &Connection, surah_id: u32) -> DbResult<Surah> {
    let mut stmt = conn.prepare(
        "SELECT id, name_ar, name_en, transliteration, revelation_type,
                verses_count, order_of_revelation, has_bismillah
         FROM surah WHERE id = ?1"
    )?;

    stmt.query_row(params![surah_id], |row| {
        Ok((
            row.get::<_, u32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, u32>(5)?,
            row.get::<_, u32>(6)?,
            row.get::<_, bool>(7)?,
        ))
    })
    .map(|(id, name_ar, name_en, transliteration, revelation_type_str,
           verses_count, order_of_revelation, has_bismillah)| {
        let revelation_type = revelation_type_str.parse::<RevelationType>()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
        Ok(Surah {
            id, name_ar, name_en, transliteration, revelation_type,
            verses_count, order_of_revelation, has_bismillah,
        })
    })
    .map_err(|_| DbError::NotFound(format!("Surah {}", surah_id)))?
}

// =============================================================================
// AYAH QUERIES
// =============================================================================

/// Return all Ayahs in a Surah ordered by ayah_number.
pub fn get_ayahs_for_surah(conn: &Connection, surah_id: u32) -> DbResult<Vec<Ayah>> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE surah_id = ?1 ORDER BY ayah_number ASC"
    )?;

    let ayahs = stmt.query_map(params![surah_id], |row| {
        Ok(Ayah {
            id:           row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            juz:          row.get(5)?,
            hizb:         row.get(6)?,
            rub_hizb:     row.get(7)?,
            manzil:       row.get(8)?,
            ruku:         row.get(9)?,
            page:         row.get(10)?,
            sajdah:       row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

/// Return all Ayahs on a given Mushaf page.
pub fn get_ayahs_for_page(conn: &Connection, page: u32) -> DbResult<Vec<Ayah>> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE page = ?1 ORDER BY id ASC"
    )?;

    let ayahs = stmt.query_map(params![page], |row| {
        Ok(Ayah {
            id:           row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            juz:          row.get(5)?,
            hizb:         row.get(6)?,
            rub_hizb:     row.get(7)?,
            manzil:       row.get(8)?,
            ruku:         row.get(9)?,
            page:         row.get(10)?,
            sajdah:       row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

/// Return all Ayahs in a given Juz.
pub fn get_ayahs_for_juz(conn: &Connection, juz: u32) -> DbResult<Vec<Ayah>> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE juz = ?1 ORDER BY id ASC"
    )?;

    let ayahs = stmt.query_map(params![juz], |row| {
        Ok(Ayah {
            id:           row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            juz:          row.get(5)?,
            hizb:         row.get(6)?,
            rub_hizb:     row.get(7)?,
            manzil:       row.get(8)?,
            ruku:         row.get(9)?,
            page:         row.get(10)?,
            sajdah:       row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

/// Return the first Ayah of a given Juz (useful for navigation).
#[allow(dead_code)] // wired up by the "Go to Juz" navigation command (PLAN.md Phase 6)
pub fn get_juz_start(conn: &Connection, juz: u32) -> DbResult<AyahRef> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, page, juz
         FROM ayah WHERE juz = ?1 ORDER BY id ASC LIMIT 1"
    )?;

    stmt.query_row(params![juz], |row| {
        Ok(AyahRef {
            id:          row.get(0)?,
            surah_id:    row.get(1)?,
            ayah_number: row.get(2)?,
            page:        row.get(3)?,
            juz:         row.get(4)?,
        })
    })
    .map_err(|_| DbError::NotFound(format!("Juz {}", juz)))
}

/// Return the first Ayah of a given Hizb.
#[allow(dead_code)] // wired up by the "Go to Hizb" navigation command (PLAN.md Phase 6)
pub fn get_hizb_start(conn: &Connection, hizb: u32) -> DbResult<AyahRef> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, page, juz
         FROM ayah WHERE hizb = ?1 ORDER BY id ASC LIMIT 1"
    )?;

    stmt.query_row(params![hizb], |row| {
        Ok(AyahRef {
            id:          row.get(0)?,
            surah_id:    row.get(1)?,
            ayah_number: row.get(2)?,
            page:        row.get(3)?,
            juz:         row.get(4)?,
        })
    })
    .map_err(|_| DbError::NotFound(format!("Hizb {}", hizb)))
}

/// Return a specific Ayah by global id.
#[allow(dead_code)] // wired up by the Bookmark list's ayah lookup (PLAN.md Phase 8)
pub fn get_ayah(conn: &Connection, ayah_id: u32) -> DbResult<Ayah> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE id = ?1"
    )?;

    stmt.query_row(params![ayah_id], |row| {
        Ok(Ayah {
            id:           row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            juz:          row.get(5)?,
            hizb:         row.get(6)?,
            rub_hizb:     row.get(7)?,
            manzil:       row.get(8)?,
            ruku:         row.get(9)?,
            page:         row.get(10)?,
            sajdah:       row.get(11)?,
        })
    })
    .map_err(|_| DbError::NotFound(format!("Ayah id={}", ayah_id)))
}

/// Return all 15 Sajdah (prostration) Ayahs.
#[allow(dead_code)] // wired up by a future Sajdah-list navigation command
pub fn get_sajdah_ayahs(conn: &Connection) -> DbResult<Vec<Ayah>> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE sajdah = 1 ORDER BY id ASC"
    )?;

    let ayahs = stmt.query_map([], |row| {
        Ok(Ayah {
            id:           row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            juz:          row.get(5)?,
            hizb:         row.get(6)?,
            rub_hizb:     row.get(7)?,
            manzil:       row.get(8)?,
            ruku:         row.get(9)?,
            page:         row.get(10)?,
            sajdah:       row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

// =============================================================================
// MUSHAF PAGE LAYOUT QUERIES
// =============================================================================

/// Return the full line-by-line layout for a single Mushaf page (1–604),
/// including every word's QCF v2 glyph string in print order.
pub fn get_page(conn: &Connection, page: u32) -> DbResult<MushafPage> {
    let mut stmt = conn.prepare(
        "SELECT pl.line_number, pl.line_type, pl.surah_id, pl.first_ayah_id, pl.last_ayah_id, pl.text,
                plw.position, plw.ayah_id, plw.word_index, plw.uthmani_text, plw.glyph_v2
         FROM page_line pl
         LEFT JOIN page_line_word plw ON plw.page_line_id = pl.id
         WHERE pl.page = ?1
         ORDER BY pl.line_number ASC, plw.position ASC",
    )?;

    let mut lines: Vec<PageLine> = Vec::new();
    let mut rows = stmt.query(params![page])?;

    while let Some(row) = rows.next()? {
        let line_number: u32 = row.get(0)?;

        let word = match row.get::<_, Option<u32>>(6)? {
            Some(position) => Some(PageLineWord {
                position,
                ayah_id: row.get(7)?,
                word_index: row.get(8)?,
                uthmani_text: row.get(9)?,
                glyph_v2: row.get(10)?,
            }),
            None => None,
        };

        match lines.last_mut() {
            Some(last) if last.line_number == line_number => {
                if let Some(w) = word {
                    last.words.push(w);
                }
            }
            _ => {
                lines.push(PageLine {
                    line_number,
                    line_type: row.get(1)?,
                    surah_id: row.get(2)?,
                    first_ayah_id: row.get(3)?,
                    last_ayah_id: row.get(4)?,
                    text: row.get(5)?,
                    words: word.into_iter().collect(),
                });
            }
        }
    }

    if lines.is_empty() {
        return Err(DbError::NotFound(format!("Page {}", page)));
    }

    Ok(MushafPage { page, lines })
}

// =============================================================================
// SEARCH QUERIES
// =============================================================================

/// Full-text search in the simplified Arabic text.
/// Returns up to `limit` results, using FTS5 snippet highlighting.
pub fn search_arabic(conn: &Connection, query: &str, limit: u32) -> DbResult<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.surah_id, a.ayah_number, a.uthmani_text, a.simple_text,
                snippet(fts_ayah, 0, '<mark>', '</mark>', '…', 20) AS snippet,
                a.page, a.juz
         FROM fts_ayah
         JOIN ayah a ON fts_ayah.rowid = a.id
         WHERE fts_ayah MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    )?;

    let results = stmt.query_map(params![query, limit], |row| {
        Ok(SearchResult {
            ayah_id:      row.get(0)?,
            surah_id:     row.get(1)?,
            ayah_number:  row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text:  row.get(4)?,
            snippet:      row.get(5)?,
            page:         row.get(6)?,
            juz:          row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

// =============================================================================
// TRANSLATION QUERIES
// =============================================================================

/// Return all available translations.
pub fn get_translations(conn: &Connection) -> DbResult<Vec<Translation>> {
    let mut stmt = conn.prepare(
        "SELECT id, language, translator, title, version, is_bundled FROM translation ORDER BY id"
    )?;

    let translations = stmt.query_map([], |row| {
        Ok(Translation {
            id:         row.get(0)?,
            language:   row.get(1)?,
            translator: row.get(2)?,
            title:      row.get(3)?,
            version:    row.get(4)?,
            is_bundled: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(translations)
}

/// Return translated text for all Ayahs in a Surah under a given translation.
#[allow(dead_code)] // wired up by Phase 10 — Translations (PLAN.md)
pub fn get_translation_for_surah(
    conn: &Connection,
    translation_id: u32,
    surah_id: u32,
) -> DbResult<Vec<TranslationAyah>> {
    let mut stmt = conn.prepare(
        "SELECT ta.translation_id, ta.ayah_id, ta.text
         FROM translation_ayah ta
         JOIN ayah a ON a.id = ta.ayah_id
         WHERE ta.translation_id = ?1 AND a.surah_id = ?2
         ORDER BY a.ayah_number ASC"
    )?;

    let rows = stmt.query_map(params![translation_id, surah_id], |row| {
        Ok(TranslationAyah {
            translation_id: row.get(0)?,
            ayah_id:        row.get(1)?,
            text:           row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

// =============================================================================
// BOOKMARK QUERIES
// =============================================================================

/// Return all bookmarks, newest first.
pub fn get_bookmarks(conn: &Connection) -> DbResult<Vec<Bookmark>> {
    let mut stmt = conn.prepare(
        "SELECT id, ayah_id, label, created_at FROM bookmark ORDER BY created_at DESC"
    )?;

    let bookmarks = stmt.query_map([], |row| {
        Ok(Bookmark {
            id:         row.get(0)?,
            ayah_id:    row.get(1)?,
            label:      row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(bookmarks)
}

/// Toggle bookmark for an Ayah. Returns `true` if bookmark now exists.
pub fn toggle_bookmark(conn: &Connection, ayah_id: u32, label: Option<&str>) -> DbResult<bool> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM bookmark WHERE ayah_id = ?1)",
        params![ayah_id],
        |row| row.get(0),
    )?;

    if exists {
        conn.execute("DELETE FROM bookmark WHERE ayah_id = ?1", params![ayah_id])?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO bookmark (ayah_id, label) VALUES (?1, ?2)",
            params![ayah_id, label],
        )?;
        Ok(true)
    }
}

// =============================================================================
// NOTES QUERIES
// =============================================================================

/// Return all notes for a given Ayah.
pub fn get_notes_for_ayah(conn: &Connection, ayah_id: u32) -> DbResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, ayah_id, content, created_at, updated_at
         FROM note WHERE ayah_id = ?1 ORDER BY created_at DESC"
    )?;

    let notes = stmt.query_map(params![ayah_id], |row| {
        Ok(Note {
            id:         row.get(0)?,
            ayah_id:    row.get(1)?,
            content:    row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(notes)
}

/// Insert or update a note.
pub fn upsert_note(conn: &Connection, note_id: Option<u32>, ayah_id: u32, content: &str) -> DbResult<u32> {
    if let Some(id) = note_id {
        conn.execute(
            "UPDATE note SET content = ?1 WHERE id = ?2",
            params![content, id],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO note (ayah_id, content) VALUES (?1, ?2)",
            params![ayah_id, content],
        )?;
        Ok(conn.last_insert_rowid() as u32)
    }
}

/// Delete a note by id.
pub fn delete_note(conn: &Connection, note_id: u32) -> DbResult<()> {
    conn.execute("DELETE FROM note WHERE id = ?1", params![note_id])?;
    Ok(())
}

// =============================================================================
// SETTINGS QUERIES
// =============================================================================

/// Read a single setting value by key.
pub fn get_setting(conn: &Connection, key: &str) -> DbResult<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .map_err(|_| DbError::NotFound(format!("Setting '{}'", key)))
}

/// Write a single setting value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Load all settings into a typed `Settings` struct.
pub fn load_settings(conn: &Connection) -> DbResult<Settings> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;

    let mut map = std::collections::HashMap::<String, String>::new();
    stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .filter_map(|r| r.ok())
        .for_each(|(k, v)| { map.insert(k, v); });

    let s = Settings {
        theme:                    map.get("theme").cloned().unwrap_or_else(|| "dark".into()),
        font:                     map.get("font").cloned().unwrap_or_else(|| "amiri-quran".into()),
        font_size:                map.get("font_size").and_then(|v| v.parse().ok()).unwrap_or(28),
        line_height:              map.get("line_height").and_then(|v| v.parse().ok()).unwrap_or(2.2),
        reader_width:             map.get("reader_width").cloned().unwrap_or_else(|| "normal".into()),
        last_read_surah_id:       map.get("last_read_surah_id").and_then(|v| v.parse().ok()).unwrap_or(1),
        last_read_ayah_id:        map.get("last_read_ayah_id").and_then(|v| v.parse().ok()).unwrap_or(1),
        preferred_translation_id: map.get("preferred_translation_id").and_then(|v| v.parse().ok()),
        show_translation:         map.get("show_translation").map(|v| v == "true").unwrap_or(true),
        show_transliteration:     map.get("show_transliteration").map(|v| v == "true").unwrap_or(false),
        show_ayah_numbers:        map.get("show_ayah_numbers").map(|v| v == "true").unwrap_or(true),
        scroll_position:          map.get("scroll_position").and_then(|v| v.parse().ok()).unwrap_or(0),
    };

    Ok(s)
}
