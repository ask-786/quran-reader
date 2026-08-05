//! Database query functions.
//! Each function takes a `&Connection` and returns a typed `DbResult`.
//! No business logic here — pure data access.

use crate::db::error::{DbError, DbResult};
use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension};

// =============================================================================
// SURAH QUERIES
// =============================================================================

/// Return all 114 Surahs ordered by Mushaf order (id ASC).
pub fn get_all_surahs(conn: &Connection) -> DbResult<Vec<Surah>> {
    let mut stmt = conn.prepare(
        "SELECT id, name_ar, name_en, transliteration, revelation_type,
                verses_count, order_of_revelation, has_bismillah
         FROM surah ORDER BY id ASC",
    )?;

    let surahs = stmt
        .query_map([], |row| {
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
            let (
                id,
                name_ar,
                name_en,
                transliteration,
                revelation_type_str,
                verses_count,
                order_of_revelation,
                has_bismillah,
            ) = r?;
            let revelation_type = revelation_type_str
                .parse::<RevelationType>()
                .map_err(rusqlite::Error::InvalidParameterName)?;
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
         FROM surah WHERE id = ?1",
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
    .map(
        |(
            id,
            name_ar,
            name_en,
            transliteration,
            revelation_type_str,
            verses_count,
            order_of_revelation,
            has_bismillah,
        )| {
            let revelation_type = revelation_type_str
                .parse::<RevelationType>()
                .map_err(rusqlite::Error::InvalidParameterName)?;
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
        },
    )
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
         FROM ayah WHERE surah_id = ?1 ORDER BY ayah_number ASC",
    )?;

    let ayahs = stmt
        .query_map(params![surah_id], |row| {
            Ok(Ayah {
                id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                juz: row.get(5)?,
                hizb: row.get(6)?,
                rub_hizb: row.get(7)?,
                manzil: row.get(8)?,
                ruku: row.get(9)?,
                page: row.get(10)?,
                sajdah: row.get(11)?,
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
         FROM ayah WHERE page = ?1 ORDER BY id ASC",
    )?;

    let ayahs = stmt
        .query_map(params![page], |row| {
            Ok(Ayah {
                id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                juz: row.get(5)?,
                hizb: row.get(6)?,
                rub_hizb: row.get(7)?,
                manzil: row.get(8)?,
                ruku: row.get(9)?,
                page: row.get(10)?,
                sajdah: row.get(11)?,
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
         FROM ayah WHERE juz = ?1 ORDER BY id ASC",
    )?;

    let ayahs = stmt
        .query_map(params![juz], |row| {
            Ok(Ayah {
                id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                juz: row.get(5)?,
                hizb: row.get(6)?,
                rub_hizb: row.get(7)?,
                manzil: row.get(8)?,
                ruku: row.get(9)?,
                page: row.get(10)?,
                sajdah: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

/// Return all Ayahs in a given Hizb.
pub fn get_ayahs_for_hizb(conn: &Connection, hizb: u32) -> DbResult<Vec<Ayah>> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE hizb = ?1 ORDER BY id ASC",
    )?;

    let ayahs = stmt
        .query_map(params![hizb], |row| {
            Ok(Ayah {
                id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                juz: row.get(5)?,
                hizb: row.get(6)?,
                rub_hizb: row.get(7)?,
                manzil: row.get(8)?,
                ruku: row.get(9)?,
                page: row.get(10)?,
                sajdah: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

/// Build an [`AyahRef`] from five consecutive columns selected in the order
/// `id, surah_id, ayah_number, page, juz`, starting at `offset`. A reading
/// session embeds two of these — one per end of the range it covers — so the
/// mapping is shared rather than written out at each call site.
fn ayah_ref(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<AyahRef> {
    Ok(AyahRef {
        id: row.get(offset)?,
        surah_id: row.get(offset + 1)?,
        ayah_number: row.get(offset + 2)?,
        page: row.get(offset + 3)?,
        juz: row.get(offset + 4)?,
    })
}

/// Return the first Ayah of a given Juz (useful for navigation).
#[allow(dead_code)] // wired up by the "Go to Juz" navigation command (PLAN.md Phase 6)
pub fn get_juz_start(conn: &Connection, juz: u32) -> DbResult<AyahRef> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, page, juz
         FROM ayah WHERE juz = ?1 ORDER BY id ASC LIMIT 1",
    )?;

    stmt.query_row(params![juz], |row| ayah_ref(row, 0))
        .map_err(|_| DbError::NotFound(format!("Juz {}", juz)))
}

/// Return the first Ayah of a given Hizb.
#[allow(dead_code)] // wired up by the "Go to Hizb" navigation command (PLAN.md Phase 6)
pub fn get_hizb_start(conn: &Connection, hizb: u32) -> DbResult<AyahRef> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, page, juz
         FROM ayah WHERE hizb = ?1 ORDER BY id ASC LIMIT 1",
    )?;

    stmt.query_row(params![hizb], |row| ayah_ref(row, 0))
        .map_err(|_| DbError::NotFound(format!("Hizb {}", hizb)))
}

/// Return a specific Ayah by global id.
#[allow(dead_code)] // wired up by the Bookmark list's ayah lookup (PLAN.md Phase 8)
pub fn get_ayah(conn: &Connection, ayah_id: u32) -> DbResult<Ayah> {
    let mut stmt = conn.prepare(
        "SELECT id, surah_id, ayah_number, uthmani_text, simple_text,
                juz, hizb, rub_hizb, manzil, ruku, page, sajdah
         FROM ayah WHERE id = ?1",
    )?;

    stmt.query_row(params![ayah_id], |row| {
        Ok(Ayah {
            id: row.get(0)?,
            surah_id: row.get(1)?,
            ayah_number: row.get(2)?,
            uthmani_text: row.get(3)?,
            simple_text: row.get(4)?,
            juz: row.get(5)?,
            hizb: row.get(6)?,
            rub_hizb: row.get(7)?,
            manzil: row.get(8)?,
            ruku: row.get(9)?,
            page: row.get(10)?,
            sajdah: row.get(11)?,
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
         FROM ayah WHERE sajdah = 1 ORDER BY id ASC",
    )?;

    let ayahs = stmt
        .query_map([], |row| {
            Ok(Ayah {
                id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                juz: row.get(5)?,
                hizb: row.get(6)?,
                rub_hizb: row.get(7)?,
                manzil: row.get(8)?,
                ruku: row.get(9)?,
                page: row.get(10)?,
                sajdah: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ayahs)
}

// =============================================================================
// MUSHAF PAGE LAYOUT QUERIES
// =============================================================================

/// Return the full line-by-line layout for a single Mushaf page (1–604),
/// including every word's QCF v2 and QCF v4 glyph strings in print order.
pub fn get_page(conn: &Connection, page: u32) -> DbResult<MushafPage> {
    let mut pages = get_pages(conn, page, page)?;
    pages
        .pop()
        .ok_or_else(|| DbError::NotFound(format!("Page {}", page)))
}

/// Same as [`get_page`] but for an inclusive page range, in one query.
///
/// A Surah spans up to 48 pages (Al-Baqara), and fetching those one at a time
/// meant 48 IPC round trips plus 48 statement preparations before the reader
/// could show anything. The layout data itself is small — the whole of
/// Al-Baqara is ~6k word rows — so the range is served as a single call.
/// Pages with no rows are skipped rather than erroring, so a range that runs
/// off the end of the Mushaf still returns what does exist.
pub fn get_pages(conn: &Connection, start: u32, end: u32) -> DbResult<Vec<MushafPage>> {
    let mut stmt = conn.prepare(
        "SELECT pl.page, pl.line_number, pl.line_type, pl.surah_id, pl.first_ayah_id, pl.last_ayah_id, pl.text,
                plw.position, plw.ayah_id, plw.word_index, plw.uthmani_text, plw.glyph_v4
         FROM page_line pl
         LEFT JOIN page_line_word plw ON plw.page_line_id = pl.id
         WHERE pl.page BETWEEN ?1 AND ?2
         ORDER BY pl.page ASC, pl.line_number ASC, plw.position ASC",
    )?;

    let mut pages: Vec<MushafPage> = Vec::new();
    let mut rows = stmt.query(params![start, end])?;

    while let Some(row) = rows.next()? {
        let page: u32 = row.get(0)?;
        let line_number: u32 = row.get(1)?;

        let word = match row.get::<_, Option<u32>>(7)? {
            Some(position) => Some(PageLineWord {
                position,
                ayah_id: row.get(8)?,
                word_index: row.get(9)?,
                uthmani_text: row.get(10)?,
                glyph_v4: row.get(11)?,
            }),
            None => None,
        };

        // Rows arrive grouped by page then line, so only the last page and its
        // last line are ever the ones being appended to.
        match pages.last_mut() {
            Some(p) if p.page == page => match p.lines.last_mut() {
                Some(last) if last.line_number == line_number => {
                    if let Some(w) = word {
                        last.words.push(w);
                    }
                }
                _ => p.lines.push(PageLine {
                    line_number,
                    line_type: row.get(2)?,
                    surah_id: row.get(3)?,
                    first_ayah_id: row.get(4)?,
                    last_ayah_id: row.get(5)?,
                    text: row.get(6)?,
                    words: word.into_iter().collect(),
                }),
            },
            _ => pages.push(MushafPage {
                page,
                lines: vec![PageLine {
                    line_number,
                    line_type: row.get(2)?,
                    surah_id: row.get(3)?,
                    first_ayah_id: row.get(4)?,
                    last_ayah_id: row.get(5)?,
                    text: row.get(6)?,
                    words: word.into_iter().collect(),
                }],
            }),
        }
    }

    Ok(pages)
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
         LIMIT ?2",
    )?;

    let results = stmt
        .query_map(params![query, limit], |row| {
            Ok(SearchResult {
                ayah_id: row.get(0)?,
                surah_id: row.get(1)?,
                ayah_number: row.get(2)?,
                uthmani_text: row.get(3)?,
                simple_text: row.get(4)?,
                snippet: row.get(5)?,
                page: row.get(6)?,
                juz: row.get(7)?,
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
        "SELECT id, language, translator, title, version, is_bundled FROM translation ORDER BY id",
    )?;

    let translations = stmt
        .query_map([], |row| {
            Ok(Translation {
                id: row.get(0)?,
                language: row.get(1)?,
                translator: row.get(2)?,
                title: row.get(3)?,
                version: row.get(4)?,
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
         ORDER BY a.ayah_number ASC",
    )?;

    let rows = stmt
        .query_map(params![translation_id, surah_id], |row| {
            Ok(TranslationAyah {
                translation_id: row.get(0)?,
                ayah_id: row.get(1)?,
                text: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

// =============================================================================
// TAFSIR QUERIES
// =============================================================================

/// Return the installed tafsir editions, bundled ones first.
pub fn get_tafsirs(conn: &Connection) -> DbResult<Vec<Tafsir>> {
    let mut stmt = conn.prepare(
        "SELECT id, language, author, title, version, is_bundled,
                slug, translator, name_native, direction, school, creed
         FROM tafsir
         ORDER BY is_bundled DESC, sort_order ASC, id ASC",
    )?;

    let tafsirs = stmt
        .query_map([], |row| {
            Ok(Tafsir {
                id: row.get(0)?,
                language: row.get(1)?,
                author: row.get(2)?,
                title: row.get(3)?,
                version: row.get(4)?,
                is_bundled: row.get(5)?,
                slug: row.get(6)?,
                translator: row.get(7)?,
                name_native: row.get(8)?,
                direction: row.get(9)?,
                school: row.get(10)?,
                creed: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tafsirs)
}

/// Commentary on a single Ayah, or `None` when this edition passes over it.
///
/// A miss is an ordinary result, not an error: al-Jalalayn glosses 6,010 of
/// the 6,236 Ayahs in its Arabic edition, saying nothing about verses that
/// need no comment. The caller shows that as an empty state rather than a
/// blank panel.
pub fn get_tafsir_for_ayah(
    conn: &Connection,
    tafsir_id: u32,
    ayah_id: u32,
) -> DbResult<Option<TafsirEntry>> {
    conn.query_row(
        "SELECT t.tafsir_id, t.ayah_id, a.surah_id, a.ayah_number, t.text,
                gs.surah_id, gs.ayah_number, ge.surah_id, ge.ayah_number
         FROM tafsir_ayah t
         JOIN ayah a       ON a.id  = t.ayah_id
         LEFT JOIN ayah gs ON gs.id = t.group_start_ayah_id
         LEFT JOIN ayah ge ON ge.id = t.group_end_ayah_id
         WHERE t.tafsir_id = ?1 AND t.ayah_id = ?2",
        params![tafsir_id, ayah_id],
        |row| {
            let verse_key = |surah: Option<u32>, ayah: Option<u32>| match (surah, ayah) {
                (Some(s), Some(a)) => Some(format!("{s}:{a}")),
                _ => None,
            };
            Ok(TafsirEntry {
                tafsir_id: row.get(0)?,
                ayah_id: row.get(1)?,
                surah_id: row.get(2)?,
                ayah_number: row.get(3)?,
                text: row.get(4)?,
                group_start_key: verse_key(row.get(5)?, row.get(6)?),
                group_end_key: verse_key(row.get(7)?, row.get(8)?),
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

// =============================================================================
// BOOKMARK QUERIES
// =============================================================================

/// Return all bookmarks, newest first.
pub fn get_bookmarks(conn: &Connection) -> DbResult<Vec<Bookmark>> {
    let mut stmt = conn
        .prepare("SELECT id, ayah_id, label, created_at FROM bookmark ORDER BY created_at DESC")?;

    let bookmarks = stmt
        .query_map([], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                ayah_id: row.get(1)?,
                label: row.get(2)?,
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
         FROM note WHERE ayah_id = ?1 ORDER BY created_at DESC",
    )?;

    let notes = stmt
        .query_map(params![ayah_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                ayah_id: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(notes)
}

/// Insert or update a note.
pub fn upsert_note(
    conn: &Connection,
    note_id: Option<u32>,
    ayah_id: u32,
    content: &str,
) -> DbResult<u32> {
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
// READING POSITION / HISTORY QUERIES
// =============================================================================

/// How long a pause can run before the next recorded position opens a fresh
/// sitting instead of extending the last one, as a SQLite time modifier. Half
/// an hour is long enough to survive a phone call or a break without splitting
/// one reading in two, and short enough that picking the Mushaf back up the
/// next morning reads as a new entry in the history.
const SESSION_IDLE_MODIFIER: &str = "-30 minutes";

/// How many sittings the history keeps. Trimmed whenever a new one opens, so
/// the table can't grow without bound over years of daily reading.
const HISTORY_LIMIT: u32 = 200;

/// The timestamp every reading row is stamped with — the same UTC ISO-8601
/// shape `bookmark`/`note` use, which is what makes the idle comparison below
/// a plain string comparison.
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ', 'now')";

/// Read a [`ReadingScope`] out of a text column, failing the row rather than
/// the whole query if the schema's CHECK constraint were ever bypassed.
fn scope_from_row(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<ReadingScope> {
    let raw: String = row.get(idx)?;
    raw.parse().map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            idx,
            rusqlite::types::Type::Text,
            Box::new(DbError::InvalidData(format!("reading scope '{}'", raw))),
        )
    })
}

/// Columns for a [`ReadingPosition`], joined onto the Ayah it points at.
const POSITION_SELECT: &str = "SELECT p.scope, p.scope_id,
            a.id, a.surah_id, a.ayah_number, a.page, a.juz,
            p.updated_at
     FROM reading_position p
     JOIN ayah a ON a.id = p.ayah_id";

fn position_from_row(row: &rusqlite::Row) -> rusqlite::Result<ReadingPosition> {
    Ok(ReadingPosition {
        scope: scope_from_row(row, 0)?,
        scope_id: row.get(1)?,
        ayah: ayah_ref(row, 2)?,
        updated_at: row.get(7)?,
    })
}

/// Record that the reader is at `ayah_id` inside `scope`/`scope_id`.
///
/// Two things happen, and they are deliberately different in kind. The
/// position is *overwritten* — there is only ever one "where I am" per range,
/// so reopening Juz 5 resumes inside Juz 5 no matter what has been read since.
/// The sitting is *extended or appended*: the newest one grows its end while
/// reading continues in the same range, and anything else — a different range,
/// or coming back after [`SESSION_IDLE_MODIFIER`] of silence — opens a new row.
/// That is what turns a stream of scroll positions into a readable history
/// instead of one ever-growing entry.
pub fn record_reading_position(
    conn: &Connection,
    scope: ReadingScope,
    scope_id: u32,
    ayah_id: u32,
) -> DbResult<()> {
    let scope = scope.as_str();
    // `unchecked_transaction` because the connection is shared behind a Mutex
    // and only reachable as `&Connection`; the lock is what serialises writers.
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        &format!(
            "INSERT INTO reading_position (scope, scope_id, ayah_id, updated_at)
             VALUES (?1, ?2, ?3, {NOW})
             ON CONFLICT(scope, scope_id) DO UPDATE
                SET ayah_id = excluded.ayah_id, updated_at = excluded.updated_at"
        ),
        params![scope, scope_id, ayah_id],
    )?;

    // Only the newest sitting is ever extended: an older one for the same
    // range belongs to a reading that has already ended, and reviving it would
    // stretch a single entry across days.
    let extended = tx.execute(
        &format!(
            "UPDATE reading_session
                SET end_ayah_id = ?3, updated_at = {NOW}
              WHERE id = (
                        SELECT id FROM reading_session
                        ORDER BY updated_at DESC, id DESC LIMIT 1
                    )
                AND scope = ?1
                AND scope_id = ?2
                AND updated_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?4)"
        ),
        params![scope, scope_id, ayah_id, SESSION_IDLE_MODIFIER],
    )?;

    if extended == 0 {
        tx.execute(
            "INSERT INTO reading_session (scope, scope_id, start_ayah_id, end_ayah_id)
             VALUES (?1, ?2, ?3, ?3)",
            params![scope, scope_id, ayah_id],
        )?;
        // Trimmed here rather than on every write: this is the only branch
        // that can push the table over the limit.
        tx.execute(
            "DELETE FROM reading_session
              WHERE id NOT IN (
                        SELECT id FROM reading_session
                        ORDER BY updated_at DESC, id DESC LIMIT ?1
                    )",
            params![HISTORY_LIMIT],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Where the reader left off inside one range, or `None` if they never opened it.
pub fn get_reading_position(
    conn: &Connection,
    scope: ReadingScope,
    scope_id: u32,
) -> DbResult<Option<ReadingPosition>> {
    let mut stmt = conn.prepare(&format!(
        "{POSITION_SELECT} WHERE p.scope = ?1 AND p.scope_id = ?2"
    ))?;

    match stmt.query_row(params![scope.as_str(), scope_id], position_from_row) {
        Ok(position) => Ok(Some(position)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// The most recently touched position across every range — "continue reading".
/// `None` on a database nothing has ever been read in.
pub fn get_last_reading_position(conn: &Connection) -> DbResult<Option<ReadingPosition>> {
    let mut stmt = conn.prepare(&format!(
        "{POSITION_SELECT} ORDER BY p.updated_at DESC, p.rowid DESC LIMIT 1"
    ))?;

    match stmt.query_row([], position_from_row) {
        Ok(position) => Ok(Some(position)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Recent sittings, newest first, each with the Ayah it started at and the one
/// it reached.
pub fn get_reading_history(conn: &Connection, limit: u32) -> DbResult<Vec<ReadingSession>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.scope, s.scope_id,
                sa.id, sa.surah_id, sa.ayah_number, sa.page, sa.juz,
                ea.id, ea.surah_id, ea.ayah_number, ea.page, ea.juz,
                s.started_at, s.updated_at
         FROM reading_session s
         JOIN ayah sa ON sa.id = s.start_ayah_id
         JOIN ayah ea ON ea.id = s.end_ayah_id
         ORDER BY s.updated_at DESC, s.id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(ReadingSession {
            id: row.get(0)?,
            scope: scope_from_row(row, 1)?,
            scope_id: row.get(2)?,
            start: ayah_ref(row, 3)?,
            end: ayah_ref(row, 8)?,
            started_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Forget every sitting. Reading *positions* deliberately survive: clearing the
/// history is about the record of what was read, not about losing your place.
pub fn clear_reading_history(conn: &Connection) -> DbResult<()> {
    conn.execute("DELETE FROM reading_session", [])?;
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
    stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?
    .filter_map(|r| r.ok())
    .for_each(|(k, v)| {
        map.insert(k, v);
    });

    let s = Settings {
        theme: map.get("theme").cloned().unwrap_or_else(|| "dark".into()),
        font: map
            .get("font")
            .cloned()
            .unwrap_or_else(|| "amiri-quran".into()),
        font_size: map
            .get("font_size")
            .and_then(|v| v.parse().ok())
            .unwrap_or(28),
        line_height: map
            .get("line_height")
            .and_then(|v| v.parse().ok())
            .unwrap_or(2.2),
        reader_width: map
            .get("reader_width")
            .cloned()
            .unwrap_or_else(|| "normal".into()),
        preferred_translation_id: map
            .get("preferred_translation_id")
            .and_then(|v| v.parse().ok()),
        show_translation: map
            .get("show_translation")
            .map(|v| v == "true")
            .unwrap_or(true),
        // Empty string is how "not chosen yet" is stored, and parse() turns it
        // into None for free.
        tafsir_id: map.get("tafsir_id").and_then(|v| v.parse().ok()),
        show_tafsir: map.get("show_tafsir").map(|v| v == "true").unwrap_or(false),
        tafsir_panel_width: map
            .get("tafsir_panel_width")
            .and_then(|v| v.parse().ok())
            .unwrap_or(420),
        show_transliteration: map
            .get("show_transliteration")
            .map(|v| v == "true")
            .unwrap_or(false),
        show_ayah_numbers: map
            .get("show_ayah_numbers")
            .map(|v| v == "true")
            .unwrap_or(true),
        app_zoom: map
            .get("app_zoom")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        // `reader_zoom_normal` falls back to the legacy unscoped `reader_zoom` key so
        // existing users keep their zoom level after the normal/focus split.
        reader_zoom_normal: map
            .get("reader_zoom_normal")
            .or_else(|| map.get("reader_zoom"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        reader_zoom_focus: map
            .get("reader_zoom_focus")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
    };

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection;

    /// A real, migrated database in a throwaway file. The reading tables have
    /// foreign keys into `ayah`, so these can't run against an empty schema.
    fn test_db(name: &str) -> (std::path::PathBuf, Connection) {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "quranreader-{name}-{}-{nonce}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let conn = connection::open(&path).unwrap();
        // Migration 006 carries the seed's `last_read_ayah_id` over as a
        // Surah 1 position; these tests are about what happens next.
        conn.execute_batch("DELETE FROM reading_position; DELETE FROM reading_session;")
            .unwrap();
        (path, conn)
    }

    fn cleanup(path: std::path::PathBuf, conn: Connection) {
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    /// The global id of an Ayah by its Surah:Ayah citation.
    fn ayah_id(conn: &Connection, surah: u32, number: u32) -> u32 {
        conn.query_row(
            "SELECT id FROM ayah WHERE surah_id = ?1 AND ayah_number = ?2",
            params![surah, number],
            |r| r.get(0),
        )
        .unwrap()
    }

    /// Age the newest sitting so the next record sees it as cold.
    fn age_newest_session(conn: &Connection, modifier: &str) {
        conn.execute(
            "UPDATE reading_session
                SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?1)
              WHERE id = (SELECT MAX(id) FROM reading_session)",
            params![modifier],
        )
        .unwrap();
    }

    /// Each range remembers its own place. This is the whole reason positions
    /// are keyed by scope: reading a Juz used to leave no trace at all, and a
    /// single Surah-shaped slot could never have held one.
    #[test]
    fn every_range_keeps_its_own_position() {
        let (path, conn) = test_db("positions");
        let baqara_142 = ayah_id(&conn, 2, 142);
        let kahf_10 = ayah_id(&conn, 18, 10);

        record_reading_position(&conn, ReadingScope::Surah, 18, kahf_10).unwrap();
        record_reading_position(&conn, ReadingScope::Juz, 2, baqara_142).unwrap();

        let surah = get_reading_position(&conn, ReadingScope::Surah, 18)
            .unwrap()
            .unwrap();
        assert_eq!(surah.ayah.id, kahf_10);
        assert_eq!((surah.ayah.surah_id, surah.ayah.ayah_number), (18, 10));

        let juz = get_reading_position(&conn, ReadingScope::Juz, 2)
            .unwrap()
            .unwrap();
        assert_eq!(juz.ayah.id, baqara_142);

        assert!(get_reading_position(&conn, ReadingScope::Hizb, 1)
            .unwrap()
            .is_none());

        // "Continue reading" is the most recently touched of them.
        let last = get_last_reading_position(&conn).unwrap().unwrap();
        assert_eq!((last.scope, last.scope_id), (ReadingScope::Juz, 2));

        cleanup(path, conn);
    }

    /// Reading on within the same range grows one entry rather than filling the
    /// history with a row per scroll — and the entry keeps the Ayah the sitting
    /// opened at, which is what makes it a range instead of a bare bookmark.
    #[test]
    fn reading_on_extends_the_open_sitting() {
        let (path, conn) = test_db("extend");
        let first = ayah_id(&conn, 2, 1);
        let middle = ayah_id(&conn, 2, 20);
        let last = ayah_id(&conn, 2, 74);

        for at in [first, middle, last] {
            record_reading_position(&conn, ReadingScope::Surah, 2, at).unwrap();
        }

        let history = get_reading_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 1, "one sitting, not one row per position");
        assert_eq!(history[0].start.id, first);
        assert_eq!(history[0].end.id, last);
        assert_eq!(history[0].start.ayah_number, 1);
        assert_eq!(history[0].end.ayah_number, 74);

        cleanup(path, conn);
    }

    /// Moving to a different range ends the sitting and opens another, so the
    /// history reads as "Al-Baqarah, then Juz 5" rather than one entry whose
    /// ends belong to different places.
    #[test]
    fn a_different_range_opens_a_new_sitting() {
        let (path, conn) = test_db("switch");
        let baqara = ayah_id(&conn, 2, 5);
        let juz_five = ayah_id(&conn, 4, 24);

        record_reading_position(&conn, ReadingScope::Surah, 2, baqara).unwrap();
        record_reading_position(&conn, ReadingScope::Juz, 5, juz_five).unwrap();

        let history = get_reading_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 2);
        // Newest first.
        assert_eq!(history[0].scope, ReadingScope::Juz);
        assert_eq!(history[0].start.id, juz_five);
        assert_eq!(history[1].scope, ReadingScope::Surah);
        assert_eq!(history[1].end.id, baqara);

        cleanup(path, conn);
    }

    /// Coming back to the same Surah the next day is a new sitting, not a
    /// continuation of the one from yesterday — otherwise a Surah read over a
    /// week would collapse into a single, ever-growing entry.
    #[test]
    fn a_cold_sitting_is_not_revived() {
        let (path, conn) = test_db("cold");
        let yesterday = ayah_id(&conn, 2, 100);
        let today = ayah_id(&conn, 2, 120);

        record_reading_position(&conn, ReadingScope::Surah, 2, yesterday).unwrap();
        age_newest_session(&conn, "-1 day");
        record_reading_position(&conn, ReadingScope::Surah, 2, today).unwrap();

        let history = get_reading_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].start.id, today, "today's sitting starts today");
        assert_eq!(history[0].end.id, today);
        assert_eq!(history[1].end.id, yesterday);

        // The position itself is overwritten, not appended: there is only ever
        // one place to resume Al-Baqarah from.
        let position = get_reading_position(&conn, ReadingScope::Surah, 2)
            .unwrap()
            .unwrap();
        assert_eq!(position.ayah.id, today);

        cleanup(path, conn);
    }

    /// A pause inside the idle window is still the same sitting — stepping away
    /// for a few minutes shouldn't split one reading in two.
    #[test]
    fn a_short_pause_stays_in_the_same_sitting() {
        let (path, conn) = test_db("pause");
        let before = ayah_id(&conn, 36, 1);
        let after = ayah_id(&conn, 36, 40);

        record_reading_position(&conn, ReadingScope::Surah, 36, before).unwrap();
        age_newest_session(&conn, "-10 minutes");
        record_reading_position(&conn, ReadingScope::Surah, 36, after).unwrap();

        let history = get_reading_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!((history[0].start.id, history[0].end.id), (before, after));

        cleanup(path, conn);
    }

    /// The history is bounded, and trimming keeps the newest rather than
    /// truncating from the wrong end.
    #[test]
    fn history_is_trimmed_to_the_newest_sittings() {
        let (path, conn) = test_db("trim");
        let overflow = HISTORY_LIMIT + 10;
        for page in 1..=overflow {
            let ayah = conn
                .query_row(
                    "SELECT id FROM ayah WHERE page = ?1 ORDER BY id LIMIT 1",
                    params![page],
                    |r| r.get::<_, u32>(0),
                )
                .unwrap();
            record_reading_position(&conn, ReadingScope::Page, page, ayah).unwrap();
        }

        let total: u32 = conn
            .query_row("SELECT COUNT(*) FROM reading_session", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, HISTORY_LIMIT);

        let history = get_reading_history(&conn, 1).unwrap();
        assert_eq!(history[0].scope_id, overflow, "the newest survives");

        // Positions are not trimmed with them — every page read is still
        // resumable, it just isn't listed as a recent sitting any more.
        let positions: u32 = conn
            .query_row("SELECT COUNT(*) FROM reading_position", [], |r| r.get(0))
            .unwrap();
        assert_eq!(positions, overflow);

        cleanup(path, conn);
    }

    /// Clearing the history is about the record of what was read, not about
    /// losing your place in it.
    #[test]
    fn clearing_history_keeps_positions() {
        let (path, conn) = test_db("clear");
        let ayah = ayah_id(&conn, 55, 13);
        record_reading_position(&conn, ReadingScope::Surah, 55, ayah).unwrap();

        clear_reading_history(&conn).unwrap();

        assert!(get_reading_history(&conn, 10).unwrap().is_empty());
        assert_eq!(
            get_last_reading_position(&conn).unwrap().unwrap().ayah.id,
            ayah
        );

        cleanup(path, conn);
    }
}
