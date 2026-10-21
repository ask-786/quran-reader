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

/// Return the installed tafsir editions.
///
/// Every one of them arrived as a downloaded pack — no commentary ships with
/// the app — so this is empty until the reader installs something, and the
/// panel has a state saying so. `is_bundled` stays in the ordering because the
/// column still means what it says; nothing sets it today.
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
///
/// Still a point query on the primary key, but grouped editions need one more
/// hop. Where a commentary covers a run of verses, the block is stored once on
/// the run's first Ayah and the rest of the run carries an empty text with
/// `group_start_ayah_id` pointing back at it — so following that pointer is
/// what turns any Ayah of the run into the block written for it. The anchor
/// row's own pointer is itself, so it costs the same join and answers with its
/// own text. Per-Ayah editions leave the column null and fall straight through
/// to `t.text`.
///
/// The alternative was to repeat the block under every Ayah it covers, which
/// is what the schema comment originally described. Measured on Ibn Kathir
/// that is 126.6 MB against 34.5 MB for the two editions, and it makes a full
/// text search return the same block once per verse of the run.
pub fn get_tafsir_for_ayah(
    conn: &Connection,
    tafsir_id: u32,
    ayah_id: u32,
) -> DbResult<Option<TafsirEntry>> {
    conn.query_row(
        "SELECT t.tafsir_id, t.ayah_id, a.surah_id, a.ayah_number,
                COALESCE(anchor.text, t.text),
                gs.surah_id, gs.ayah_number, ge.surah_id, ge.ayah_number
         FROM tafsir_ayah t
         JOIN ayah a       ON a.id  = t.ayah_id
         LEFT JOIN ayah gs ON gs.id = t.group_start_ayah_id
         LEFT JOIN ayah ge ON ge.id = t.group_end_ayah_id
         LEFT JOIN tafsir_ayah anchor
                ON anchor.tafsir_id = t.tafsir_id
               AND anchor.ayah_id   = t.group_start_ayah_id
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
// ROUTINE QUERIES
// =============================================================================

/// The Surahs of each routine, in step order, keyed by routine id.
fn routine_items(conn: &Connection) -> DbResult<std::collections::HashMap<u32, Vec<u32>>> {
    let mut stmt = conn
        .prepare("SELECT routine_id, surah_id FROM routine_item ORDER BY routine_id, position")?;
    let mut items = std::collections::HashMap::<u32, Vec<u32>>::new();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?)))?;
    for row in rows {
        let (routine_id, surah_id) = row?;
        items.entry(routine_id).or_default().push(surah_id);
    }
    Ok(items)
}

/// Every routine, in the order the reader arranged them.
pub fn get_routines(conn: &Connection) -> DbResult<Vec<Routine>> {
    let mut items = routine_items(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, run_step, run_started_at FROM routine ORDER BY sort_order, id",
    )?;
    let routines = stmt
        .query_map([], |row| {
            Ok(Routine {
                id: row.get(0)?,
                name: row.get(1)?,
                surah_ids: Vec::new(),
                run_step: row.get(2)?,
                run_started_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(routines
        .into_iter()
        .map(|mut r| {
            r.surah_ids = items.remove(&r.id).unwrap_or_default();
            r
        })
        .collect())
}

/// One routine by id. Returns `DbError::NotFound` if it has been deleted.
pub fn get_routine(conn: &Connection, routine_id: u32) -> DbResult<Routine> {
    get_routines(conn)?
        .into_iter()
        .find(|r| r.id == routine_id)
        .ok_or_else(|| DbError::NotFound(format!("routine {}", routine_id)))
}

/// Create a routine (`routine_id` of `None`) or replace an existing one's name
/// and Surahs, returning it as stored.
///
/// The Surahs are replaced wholesale rather than moved row by row: a routine is
/// a handful of items, and one delete-and-insert is far simpler than reconciling
/// positions under a primary key that includes them. A changed list also drops
/// the run in progress, because its step index would now point at a different
/// Surah; a rename alone leaves it be.
pub fn save_routine(
    conn: &Connection,
    routine_id: Option<u32>,
    name: &str,
    surah_ids: &[u32],
) -> DbResult<Routine> {
    let name = name.trim();
    if name.is_empty() {
        return Err(DbError::InvalidData("a routine needs a name".into()));
    }
    if surah_ids.is_empty() {
        return Err(DbError::InvalidData(
            "a routine needs at least one Surah".into(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for &id in surah_ids {
        if !(1..=114).contains(&id) {
            return Err(DbError::InvalidData(format!("Surah {} doesn't exist", id)));
        }
        if !seen.insert(id) {
            return Err(DbError::InvalidData(format!(
                "Surah {} is in the routine twice",
                id
            )));
        }
    }

    let tx = conn.unchecked_transaction()?;

    let id = match routine_id {
        Some(id) => {
            let changed = tx.execute(
                &format!("UPDATE routine SET name = ?2, updated_at = {NOW} WHERE id = ?1"),
                params![id, name],
            )?;
            if changed == 0 {
                return Err(DbError::NotFound(format!("routine {}", id)));
            }
            id
        }
        None => {
            tx.execute(
                "INSERT INTO routine (name, sort_order)
                 VALUES (?1, (SELECT COALESCE(MAX(sort_order), -1) + 1 FROM routine))",
                params![name],
            )?;
            tx.last_insert_rowid() as u32
        }
    };

    let existing: Vec<u32> = tx
        .prepare("SELECT surah_id FROM routine_item WHERE routine_id = ?1 ORDER BY position")?
        .query_map(params![id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    if existing != surah_ids {
        tx.execute(
            "DELETE FROM routine_item WHERE routine_id = ?1",
            params![id],
        )?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO routine_item (routine_id, position, surah_id) VALUES (?1, ?2, ?3)",
            )?;
            for (position, surah_id) in surah_ids.iter().enumerate() {
                insert.execute(params![id, position as u32, surah_id])?;
            }
        }
        tx.execute(
            "UPDATE routine SET run_step = NULL, run_started_at = NULL WHERE id = ?1",
            params![id],
        )?;
    }

    tx.commit()?;
    get_routine(conn, id)
}

/// Delete a routine; its items go with it through the foreign key's cascade.
pub fn delete_routine(conn: &Connection, routine_id: u32) -> DbResult<()> {
    conn.execute("DELETE FROM routine WHERE id = ?1", params![routine_id])?;
    Ok(())
}

/// Put the routines in the given order. Ids not in the list keep their old
/// `sort_order` and so sort after the ones that are listed.
pub fn reorder_routines(conn: &Connection, routine_ids: &[u32]) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    {
        let mut update = tx.prepare("UPDATE routine SET sort_order = ?2 WHERE id = ?1")?;
        for (order, id) in routine_ids.iter().enumerate() {
            update.execute(params![id, order as i64])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Move a routine's run to `step`, returning the routine as stored.
///
/// `new_run` stamps the run as starting now — the caller decides when that is,
/// since it is the one that knows when the reader's day turned over. `step` may
/// equal the number of Surahs, which is how a finished run is recorded.
pub fn set_routine_step(
    conn: &Connection,
    routine_id: u32,
    step: u32,
    new_run: bool,
) -> DbResult<Routine> {
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM routine_item WHERE routine_id = ?1",
        params![routine_id],
        |row| row.get(0),
    )?;
    if step > count {
        return Err(DbError::InvalidData(format!(
            "step {} is past the end of a {}-Surah routine",
            step, count
        )));
    }

    let changed = if new_run {
        conn.execute(
            &format!("UPDATE routine SET run_step = ?2, run_started_at = {NOW} WHERE id = ?1"),
            params![routine_id, step],
        )?
    } else {
        conn.execute(
            "UPDATE routine SET run_step = ?2 WHERE id = ?1",
            params![routine_id, step],
        )?
    };
    if changed == 0 {
        return Err(DbError::NotFound(format!("routine {}", routine_id)));
    }

    get_routine(conn, routine_id)
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
        tafsir_card_width: map
            .get("tafsir_card_width")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        tafsir_card_height: map
            .get("tafsir_card_height")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        tafsir_zoom: map
            .get("tafsir_zoom")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        // Absent in databases predating the popover, which is why the default
        // lives here as well as in schema.sql — no migration is needed for a
        // settings key.
        tafsir_view: map
            .get("tafsir_view")
            .cloned()
            .unwrap_or_else(|| "popover".into()),
        tafsir_click: map
            .get("tafsir_click")
            .map(|v| v == "true")
            .unwrap_or(false),
        show_transliteration: map
            .get("show_transliteration")
            .map(|v| v == "true")
            .unwrap_or(false),
        show_ayah_numbers: map
            .get("show_ayah_numbers")
            .map(|v| v == "true")
            .unwrap_or(true),
        // Absent in databases predating range focus; like `tafsir_view` above,
        // a settings key needs no migration to gain a default.
        range_focus: map
            .get("range_focus")
            .cloned()
            .unwrap_or_else(|| "trim".into()),
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
        // Audio. Absent in every database that predates migration 009, and the
        // defaults here are what those installs get — same arrangement as
        // `tafsir_view`. An empty `reciter_id` parses to None, which is "no
        // reciter chosen" and therefore "no network".
        reciter_id: map.get("reciter_id").and_then(|v| v.parse().ok()),
        audio_bitrate: map
            .get("audio_bitrate")
            .and_then(|v| v.parse().ok())
            .unwrap_or(64),
        audio_repeat_mode: map
            .get("audio_repeat_mode")
            .cloned()
            .unwrap_or_else(|| "off".into()),
        audio_repeat_count: map
            .get("audio_repeat_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(3),
        audio_repeat_pause_ms: map
            .get("audio_repeat_pause_ms")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        audio_playback_rate: map
            .get("audio_playback_rate")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        audio_follow: map.get("audio_follow").map(|v| v == "true").unwrap_or(true),
        audio_volume: map
            .get("audio_volume")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        audio_downloads_allowed: map
            .get("audio_downloads_allowed")
            .map(|v| v == "true")
            .unwrap_or(false),
        audio_bar_surah: map
            .get("audio_bar_surah")
            .map(|v| v == "true")
            .unwrap_or(true),
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

    /// Several routines, each keeping its Surahs in the order they were given
    /// and the routines themselves in the order they were made.
    #[test]
    fn routines_keep_their_order() {
        let (path, conn) = test_db("routines");

        let morning = save_routine(&conn, None, " Morning ", &[36, 56, 67]).unwrap();
        let night = save_routine(&conn, None, "Before sleep", &[67, 32]).unwrap();
        assert_eq!(morning.name, "Morning", "names are trimmed");
        assert_eq!(morning.surah_ids, vec![36, 56, 67]);
        assert_eq!(morning.run_step, None);

        let all = get_routines(&conn).unwrap();
        let names: Vec<_> = all.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Morning", "Before sleep"]);
        // The same Surah may sit in two routines.
        assert_eq!(all[1].surah_ids, vec![67, 32]);

        reorder_routines(&conn, &[night.id, morning.id]).unwrap();
        let names: Vec<_> = get_routines(&conn)
            .unwrap()
            .into_iter()
            .map(|r| r.name)
            .collect();
        assert_eq!(names, ["Before sleep", "Morning"]);

        cleanup(path, conn);
    }

    /// Each routine's run is its own, a rename leaves it alone, and changing the
    /// Surahs drops it — its step would otherwise point at a different Surah.
    #[test]
    fn a_routine_run_survives_a_rename_but_not_a_new_list() {
        let (path, conn) = test_db("routine-run");

        let morning = save_routine(&conn, None, "Morning", &[36, 56, 67]).unwrap();
        let night = save_routine(&conn, None, "Night", &[67, 32]).unwrap();

        let started = set_routine_step(&conn, morning.id, 0, true).unwrap();
        assert_eq!(started.run_step, Some(0));
        assert!(started.run_started_at.is_some());
        let stepped = set_routine_step(&conn, morning.id, 2, false).unwrap();
        assert_eq!(stepped.run_step, Some(2));
        assert_eq!(stepped.run_started_at, started.run_started_at);

        // Starting another routine doesn't touch this one's progress.
        set_routine_step(&conn, night.id, 1, true).unwrap();
        assert_eq!(get_routine(&conn, morning.id).unwrap().run_step, Some(2));

        let renamed = save_routine(&conn, Some(morning.id), "Fajr", &[36, 56, 67]).unwrap();
        assert_eq!(renamed.run_step, Some(2));

        let reordered = save_routine(&conn, Some(morning.id), "Fajr", &[67, 36, 56]).unwrap();
        assert_eq!(reordered.surah_ids, vec![67, 36, 56]);
        assert_eq!(reordered.run_step, None);
        assert_eq!(reordered.run_started_at, None);

        // Done is one past the last step; anything further is an error.
        assert_eq!(
            set_routine_step(&conn, morning.id, 3, false)
                .unwrap()
                .run_step,
            Some(3)
        );
        assert!(set_routine_step(&conn, morning.id, 4, false).is_err());

        cleanup(path, conn);
    }

    #[test]
    fn a_routine_rejects_bad_lists() {
        let (path, conn) = test_db("routine-invalid");

        assert!(save_routine(&conn, None, "  ", &[1]).is_err());
        assert!(save_routine(&conn, None, "Empty", &[]).is_err());
        assert!(save_routine(&conn, None, "Twice", &[18, 18]).is_err());
        assert!(save_routine(&conn, None, "Nowhere", &[115]).is_err());
        assert!(save_routine(&conn, Some(999), "Gone", &[1]).is_err());
        assert!(get_routines(&conn).unwrap().is_empty());

        cleanup(path, conn);
    }

    #[test]
    fn deleting_a_routine_takes_its_items() {
        let (path, conn) = test_db("routine-delete");

        let r = save_routine(&conn, None, "Friday", &[18]).unwrap();
        delete_routine(&conn, r.id).unwrap();

        assert!(get_routines(&conn).unwrap().is_empty());
        let items: u32 = conn
            .query_row("SELECT COUNT(*) FROM routine_item", [], |row| row.get(0))
            .unwrap();
        assert_eq!(items, 0);

        cleanup(path, conn);
    }
}
