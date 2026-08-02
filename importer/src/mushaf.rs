/// Mushaf page layout — imports the line-by-line Madani print layout so a
/// page can be rendered with the same line breaks and word placement as the
/// printed Mushaf.
///
/// Data source: zonetecde/mushaf-layout (ISC license), 604 page JSON files.
/// Glyphs are QCF v2 (King Fahd Complex, Uthman Taha calligraphy); the
/// matching font files are vendored separately (see scripts/vendor-mushaf-fonts.sh).
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const LAYOUT_RAW_BASE: &str =
    "https://raw.githubusercontent.com/zonetecde/mushaf-layout/main/mushaf";

#[derive(Debug, Deserialize)]
pub struct PageJson {
    page: u32,
    lines: Vec<LineJson>,
}

#[derive(Debug, Deserialize)]
struct LineJson {
    line: u32,
    #[serde(rename = "type")]
    line_type: String, // "surah-header" | "basmala" | "text"
    // Note: `surah-header` lines also carry `surah`/`text` fields, but we
    // don't trust them (see comment at the "surah-header" match arm below)
    // so they aren't even deserialized here.
    #[serde(default, rename = "verseRange")]
    verse_range: Option<String>,
    #[serde(default)]
    words: Option<Vec<WordJson>>,
}

// This source's `qpcV2` glyph strings are deliberately not deserialized. It
// remains the authority for *layout* — which words fall on which line, and
// where the lines break — but the glyphs themselves now come from the v4 pass
// below, so reading qpcV2 would only be to throw it away.
#[derive(Debug, Deserialize)]
struct WordJson {
    location: String, // "surah:ayah:word", e.g. "2:255:1"
    word: String,
}

/// Download all 604 page layout files. One request per page — this is a
/// one-time developer-run step, not something end users' installs do.
pub fn fetch_all_pages() -> Result<Vec<PageJson>> {
    let mut pages = Vec::with_capacity(604);

    for n in 1..=604u32 {
        let url = format!("{LAYOUT_RAW_BASE}/page-{n:03}.json");
        let body = crate::fetch::get_pub(&url)
            .with_context(|| format!("Fetching mushaf layout page {n}"))?;
        let page: PageJson = serde_json::from_str(&body)
            .with_context(|| format!("Parsing mushaf layout page {n}"))?;
        pages.push(page);

        if n % 50 == 0 {
            log::info!("      … layout {n}/604");
        }
    }

    Ok(pages)
}

/// Insert the fetched page layout into an existing database (opened fresh at
/// `db_path`, after Surahs/Ayahs are already present — needed to resolve
/// `surah:ayah` references to `ayah.id`).
pub fn write_mushaf_layout(db_path: &Path, pages: &[PageJson]) -> Result<()> {
    let conn = Connection::open(db_path).context("Opening database for mushaf layout")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // (surah_id, ayah_number) -> ayah.id, built once from the already-inserted Ayahs.
    let mut ayah_ids: HashMap<(u32, u32), i64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, surah_id, ayah_number FROM ayah")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })?;
        for row in rows {
            let (id, surah_id, ayah_number) = row?;
            ayah_ids.insert((surah_id, ayah_number), id);
        }
    }

    let basmala_text: String = conn.query_row(
        "SELECT uthmani_text FROM ayah WHERE surah_id = 1 AND ayah_number = 1",
        [],
        |r| r.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;
    let mut line_count = 0u32;
    let mut word_count = 0u32;

    {
        let mut line_stmt = tx.prepare(
            "INSERT INTO page_line
             (page, line_number, line_type, surah_id, first_ayah_id, last_ayah_id, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        let mut word_stmt = tx.prepare(
            "INSERT INTO page_line_word
             (page_line_id, position, ayah_id, word_index, uthmani_text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for page in pages {
            for line in &page.lines {
                match line.line_type.as_str() {
                    // The source dataset's own surah-header lines are unreliable: some
                    // surahs are missing one entirely, and a few carry a stray duplicate
                    // with the wrong surah attached to a boundary page (spot-checked
                    // against the actual Mushaf — e.g. a bogus "surah 3" header on page 76,
                    // the page *after* Aal-Imran already ended). We skip them here and
                    // synthesize all 114 headers below from our own validated Surah/Ayah
                    // data instead, which we already trust.
                    "surah-header" => {}

                    "basmala" => {
                        let page_line_id = {
                            line_stmt.execute(params![
                                page.page,
                                line.line,
                                "basmala",
                                Option::<i64>::None,
                                Option::<i64>::None,
                                Option::<i64>::None,
                                Option::<String>::None,
                            ])?;
                            tx.last_insert_rowid()
                        };
                        line_count += 1;

                        // A basmala line's glyph used to be validated here, via
                        // the source's own qpcV2 field. The v4 pass's closing
                        // "still have no glyph_v4" count now covers this — and
                        // covers it better, since it checks the glyph actually
                        // being rendered rather than the one being discarded.
                        word_stmt.execute(params![
                            page_line_id,
                            0i64,
                            Option::<i64>::None,
                            Option::<i64>::None,
                            basmala_text,
                        ])?;
                        word_count += 1;
                    }

                    "text" => {
                        let (first_ayah_id, last_ayah_id) =
                            verse_range_bounds(line.verse_range.as_deref(), &ayah_ids)
                                .with_context(|| {
                                    format!("page {} line {}", page.page, line.line)
                                })?;

                        line_stmt.execute(params![
                            page.page,
                            line.line,
                            "text",
                            Option::<i64>::None,
                            first_ayah_id,
                            last_ayah_id,
                            Option::<String>::None,
                        ])?;
                        let page_line_id = tx.last_insert_rowid();
                        line_count += 1;

                        for (position, w) in line.words.as_deref().unwrap_or(&[]).iter().enumerate()
                        {
                            let (surah_id, ayah_number, word_index) = parse_location(&w.location)
                                .with_context(|| {
                                format!("page {} line {}", page.page, line.line)
                            })?;
                            let ayah_id = ayah_ids.get(&(surah_id, ayah_number)).copied();

                            word_stmt.execute(params![
                                page_line_id,
                                position as i64,
                                ayah_id,
                                word_index,
                                w.word,
                            ])?;
                            word_count += 1;
                        }
                    }

                    other => bail!(
                        "page {} line {}: unknown line type `{other}`",
                        page.page,
                        line.line
                    ),
                }
            }
        }
    }

    let header_count = synthesize_surah_headers(&tx)?;
    line_count += header_count;
    log::info!("      Synthesized {header_count} surah_header lines from Surah/Ayah data");

    warn_missing_basmala(&tx)?;

    tx.commit()
        .context("Committing mushaf layout transaction")?;

    log::info!("      Inserted {line_count} page lines, {word_count} words");

    Ok(())
}

/// Insert exactly one `surah_header` line where each of the 114 Surahs
/// begins, using our own already-validated Surah/Ayah data rather than the
/// source dataset's unreliable `surah-header` lines. The header is anchored
/// immediately before that Surah's own opening Basmala (or its opening text
/// line, for Surahs 1 and 9, which have none) — not unconditionally at the
/// top of the page — since several pages (short Surahs near the end of the
/// Mushaf) hold more than one Surah's opening on the same page. Existing
/// lines from the anchor onward are shifted down by one to make room.
fn synthesize_surah_headers(tx: &rusqlite::Transaction) -> Result<u32> {
    let mut stmt = tx.prepare(
        "SELECT s.id, s.name_ar, a.page
         FROM surah s
         JOIN ayah a ON a.surah_id = s.id AND a.ayah_number = 1
         ORDER BY s.id ASC",
    )?;
    let starts: Vec<(u32, String, u32)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;

    for (surah_id, name_ar, page) in &starts {
        let anchor = header_anchor_line(tx, *page, *surah_id)?;

        // Two-step offset avoids transiently colliding with the UNIQUE(page, line_number)
        // index while lines at/after the anchor shift down by one.
        tx.execute(
            "UPDATE page_line SET line_number = line_number + 1000
             WHERE page = ?1 AND line_number >= ?2",
            params![page, anchor],
        )?;
        tx.execute(
            "UPDATE page_line SET line_number = line_number - 999
             WHERE page = ?1 AND line_number >= ?2 + 1000",
            params![page, anchor],
        )?;
        tx.execute(
            "INSERT INTO page_line (page, line_number, line_type, surah_id, first_ayah_id, last_ayah_id, text)
             VALUES (?1, ?2, 'surah_header', ?3, NULL, NULL, ?4)",
            params![page, anchor, surah_id, name_ar],
        )?;
    }

    Ok(starts.len() as u32)
}

/// The `line_number` a new header for `surah_id` on `page` should occupy:
/// right before that Surah's Basmala line if it has one on this page,
/// otherwise right before its opening text line.
fn header_anchor_line(tx: &rusqlite::Transaction, page: u32, surah_id: u32) -> Result<u32> {
    let text_line: u32 = tx
        .query_row(
            "SELECT pl.line_number
             FROM page_line pl
             JOIN ayah a ON a.surah_id = ?2 AND a.ayah_number = 1
             WHERE pl.page = ?1 AND pl.line_type = 'text'
               AND pl.first_ayah_id <= a.id AND pl.last_ayah_id >= a.id
             ORDER BY pl.line_number ASC LIMIT 1",
            params![page, surah_id],
            |r| r.get(0),
        )
        .with_context(|| format!("Surah {surah_id}: no opening text line found on page {page}"))?;

    let preceding_basmala: Option<u32> = match text_line.checked_sub(1) {
        Some(prev) => tx
            .query_row(
                "SELECT line_number FROM page_line
                 WHERE page = ?1 AND line_number = ?2 AND line_type = 'basmala'",
                params![page, prev],
                |r| r.get(0),
            )
            .optional()?,
        None => None,
    };

    Ok(preceding_basmala.unwrap_or(text_line))
}

/// Every Surah except 1 (Al-Fatihah, whose Basmala IS Ayah 1) and 9 (At-Tawbah,
/// which has none) should show a basmala line on its opening page. The source
/// dataset is missing the glyph string for a couple of these outright — unlike
/// the header, we can't safely synthesize a replacement (QCF v2 glyph
/// codepoints are specific to each page's own font file, so borrowing a glyph
/// string from a different page would render as the wrong shapes). We just
/// surface the gap loudly instead of silently shipping bad or absent data.
fn warn_missing_basmala(tx: &rusqlite::Transaction) -> Result<()> {
    let mut stmt = tx.prepare(
        "SELECT s.id, s.transliteration, a.page
         FROM surah s
         JOIN ayah a ON a.surah_id = s.id AND a.ayah_number = 1
         WHERE s.id NOT IN (1, 9)
           AND NOT EXISTS (
             SELECT 1 FROM page_line pl WHERE pl.page = a.page AND pl.line_type = 'basmala'
           )
         ORDER BY s.id ASC",
    )?;
    let missing: Vec<(u32, String, u32)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;

    for (surah_id, name, page) in &missing {
        log::warn!(
            "      No basmala glyphs in source data for Surah {surah_id} ({name}), page {page}"
        );
    }

    Ok(())
}

/// One-off repair for a database built before the header-anchoring fix
/// above: clears every synthesized `surah_header` line, closes the gaps
/// their (wrong) insertion left behind, then re-synthesizes them at the
/// correct anchors. Content lines' relative order on each page was never
/// disturbed by the original bug — only where headers landed — so this is
/// safe to run against already-imported data without re-fetching anything.
pub fn repair_surah_headers(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path).context("Opening database for header repair")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let tx = conn.unchecked_transaction()?;

    let removed = tx.execute("DELETE FROM page_line WHERE line_type = 'surah_header'", [])?;
    log::info!("      Removed {removed} existing surah_header lines");

    let pages: Vec<u32> = {
        let mut stmt = tx.prepare("SELECT DISTINCT page FROM page_line ORDER BY page")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    for page in pages {
        let ids: Vec<i64> = {
            let mut stmt =
                tx.prepare("SELECT id FROM page_line WHERE page = ?1 ORDER BY line_number ASC")?;
            let rows = stmt.query_map(params![page], |r| r.get(0))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        // Two-step offset dodges the UNIQUE(page, line_number) index while
        // closing the gaps left by the removed headers.
        for (i, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE page_line SET line_number = ?1 + 10000 WHERE id = ?2",
                params![i as i64 + 1, id],
            )?;
        }
        for id in &ids {
            tx.execute(
                "UPDATE page_line SET line_number = line_number - 10000 WHERE id = ?1",
                params![id],
            )?;
        }
    }

    let header_count = synthesize_surah_headers(&tx)?;
    log::info!("      Re-synthesized {header_count} surah_header lines");

    tx.commit().context("Committing header repair")?;
    Ok(())
}

/// Parse a word `location` field: `"surah:ayah:word"` (all 1-based).
fn parse_location(loc: &str) -> Result<(u32, u32, u32)> {
    let parts: Vec<&str> = loc.split(':').collect();
    let [s, a, w] = parts[..] else {
        bail!("Malformed word location `{loc}` (expected surah:ayah:word)");
    };
    Ok((s.parse()?, a.parse()?, w.parse()?))
}

/// Resolve a `verseRange` field (`"2:253-2:254"`) to `(first_ayah_id, last_ayah_id)`.
fn verse_range_bounds(
    range: Option<&str>,
    ayah_ids: &HashMap<(u32, u32), i64>,
) -> Result<(Option<i64>, Option<i64>)> {
    let Some(range) = range else {
        return Ok((None, None));
    };
    let (a, b) = range
        .split_once('-')
        .with_context(|| format!("Malformed verseRange `{range}`"))?;
    let (sid1, an1) = parse_surah_ayah(a)?;
    let (sid2, an2) = parse_surah_ayah(b)?;
    Ok((
        ayah_ids.get(&(sid1, an1)).copied(),
        ayah_ids.get(&(sid2, an2)).copied(),
    ))
}

fn parse_surah_ayah(s: &str) -> Result<(u32, u32)> {
    let (a, b) = s
        .split_once(':')
        .with_context(|| format!("Malformed surah:ayah `{s}`"))?;
    Ok((a.parse()?, b.parse()?))
}

// =============================================================================
// QCF v4 — see docs/qcf-v4-font-migration-plan.md
//
// v4 ships its own per-page JSON with a `code`/`font` pair per word instead of
// v2's `qpcV2` glyph string, and a 47-file font-group set instead of 604
// per-page fonts. The v2 layout above stays the trusted source for line
// breaks, word boundaries, and surah-header synthesis (independent of font
// version — see the migration plan's "Scope boundary"); this section only
// *attaches* a v4 glyph to each `page_line_word` row v2 already created.
// =============================================================================

const V4_RAW_BASE: &str = "https://raw.githubusercontent.com/MohamadHajjRabee/quran-qcf4/main";

#[derive(Debug, Deserialize)]
pub struct PageV4Json {
    lines: Vec<LineV4Json>,
}

#[derive(Debug, Deserialize)]
struct LineV4Json {
    words: Vec<WordV4Json>,
}

#[derive(Debug, Deserialize)]
struct WordV4Json {
    code: u32,
    #[serde(rename = "type")]
    word_type: String, // "word" | "end" | "surah_header" | "bismillah" | "quarter"
    #[serde(default)]
    verse_key: Option<String>,
    #[serde(default)]
    position: Option<u32>, // 1-based within the ayah — same numbering as v2's word_index
    #[serde(default)]
    sura: Option<u32>,
}

/// Load all 604 QCF v4 page-layout files from a local directory of
/// `001.json` … `604.json` — the layout the upstream repo and its npm package
/// (`quran-qcf4`) both use, so extracting either one gives a usable directory.
///
/// Preferred over `fetch_all_pages_v4` when you have the files: one 604-file
/// read beats 604 sequential HTTPS requests, and it makes the pass repeatable
/// offline rather than dependent on a CDN staying reachable for ten minutes.
pub fn load_all_pages_v4(dir: &Path) -> Result<Vec<PageV4Json>> {
    let mut pages = Vec::with_capacity(604);

    for n in 1..=604u32 {
        let path = dir.join(format!("{n:03}.json"));
        let body = std::fs::read_to_string(&path)
            .with_context(|| format!("Reading QCF v4 layout {}", path.display()))?;
        let page: PageV4Json = serde_json::from_str(&body)
            .with_context(|| format!("Parsing QCF v4 layout {}", path.display()))?;
        pages.push(page);
    }

    Ok(pages)
}

/// Download all 604 QCF v4 page-layout files. The JSON itself is MIT
/// licensed; the fonts it references are not (see THIRD-PARTY-NOTICES.md).
pub fn fetch_all_pages_v4() -> Result<Vec<PageV4Json>> {
    let mut pages = Vec::with_capacity(604);

    for n in 1..=604u32 {
        let url = format!("{V4_RAW_BASE}/pages/{n:03}.json");
        let body = crate::fetch::get_pub(&url)
            .with_context(|| format!("Fetching QCF v4 layout page {n}"))?;
        let page: PageV4Json = serde_json::from_str(&body)
            .with_context(|| format!("Parsing QCF v4 layout page {n}"))?;
        pages.push(page);

        if n % 50 == 0 {
            log::info!("      … v4 layout {n}/604");
        }
    }

    Ok(pages)
}

/// Populate `page_line_word.glyph_v4` on the rows the v2 import already
/// created. Doesn't touch line structure, `uthmani_text`, `ayah_id`, or
/// `word_index` — those stay v2's, per the migration plan's scope boundary.
///
/// Matching is purely by `(ayah_id, word_index)`: v4's `position` numbers a
/// word 1-based within its ayah, exactly like v2's `word_index` (confirmed
/// against Ayat al-Kursi, 2:255 — 50 words across 6 lines, `position` running
/// 1..50 before its `end` marker at 51). This is more robust than matching by
/// page/line number, since it doesn't assume the two sources break lines
/// identically.
///
/// A v4 `"end"` verse-marker entry is appended (space-joined) onto the glyph
/// of the word at `position - 1`, mirroring how v2 already embeds its own end
/// marker in the last word's `qpcV2` string two-glyphs-in-one-row — so v2 and
/// v4 glyphs stay one row per `(ayah_id, word_index)`, never diverging in row
/// count.
///
/// v4's `"bismillah"` entries (one whole-phrase glyph per Surah-opening page,
/// like v2's `basmala` line) are matched to the existing basmala row via the
/// Surah of the text line immediately following it on the same page —
/// basmala rows carry no `ayah_id`/`surah_id` of their own.
///
/// v4's `"quarter"` (rub-el-hizb ۞) entries carry a `verse_key` but no
/// `position` of their own — the ornament is glued to the word that follows
/// it on the line, so it's resolved through that word and *prepended* onto
/// its glyph, the mirror of how `"end"` appends onto the word before it. v2
/// never rendered these at all, which is why `uthmani_text` has long read
/// `۞ سَيَقُولُ` while the glyph beside it rendered only the word.
///
/// The prepends run in a second pass, after every `"word"` entry has been
/// written. A quarter precedes its word in the line, so applying it inline
/// would be overwritten moments later by that word's own `UPDATE` — and
/// deferring keeps the whole function idempotent, since a re-run resets each
/// glyph to the bare word before anything is attached to it again.
///
/// `"surah_header"` entries are still skipped: the banner text is
/// live-rendered from `surah.name_ar`, not from page-glyph data (see
/// `SurahHeader.svelte`), for both font versions.
pub fn write_glyph_v4(db_path: &Path, pages: &[PageV4Json]) -> Result<()> {
    let conn = Connection::open(db_path).context("Opening database for QCF v4 glyphs")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let mut ayah_ids: HashMap<(u32, u32), i64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, surah_id, ayah_number FROM ayah")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })?;
        for row in rows {
            let (id, surah_id, ayah_number) = row?;
            ayah_ids.insert((surah_id, ayah_number), id);
        }
    }

    // surah_id -> the id of that surah's basmala page_line_word row, found via
    // the text line immediately following each basmala line on the same page
    // (the reverse direction of header_anchor_line's own reasoning above).
    let mut surah_basmala_word: HashMap<u32, i64> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT a.surah_id, bw.id
             FROM page_line bl
             JOIN page_line_word bw ON bw.page_line_id = bl.id
             JOIN page_line nl ON nl.page = bl.page AND nl.line_number = bl.line_number + 1 AND nl.line_type = 'text'
             JOIN ayah a ON a.id = nl.first_ayah_id
             WHERE bl.line_type = 'basmala'",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (surah_id, word_id) = row?;
            surah_basmala_word.insert(surah_id, word_id);
        }
    }

    let tx = conn.unchecked_transaction()?;
    let mut word_updates = 0u32;
    let mut basmala_updates = 0u32;
    let mut quarter_updates = 0u32;
    let mut unmatched = 0u32;
    // (glyph, ayah_id, word_index) for the second pass — see the doc comment.
    let mut quarters: Vec<(String, i64, u32)> = Vec::new();

    {
        let mut update_word = tx.prepare(
            "UPDATE page_line_word SET glyph_v4 = ?1 WHERE ayah_id = ?2 AND word_index = ?3",
        )?;
        let mut append_end = tx.prepare(
            "UPDATE page_line_word SET glyph_v4 = glyph_v4 || ' ' || ?1
             WHERE ayah_id = ?2 AND word_index = ?3",
        )?;
        let mut update_basmala =
            tx.prepare("UPDATE page_line_word SET glyph_v4 = ?1 WHERE id = ?2")?;

        for page in pages {
            for line in &page.lines {
                for (i, w) in line.words.iter().enumerate() {
                    let Some(ch) = char::from_u32(w.code) else {
                        continue;
                    };
                    let glyph = ch.to_string();

                    match w.word_type.as_str() {
                        "word" => {
                            let Some((ayah_id, position)) = resolve_ayah_position(w, &ayah_ids)
                            else {
                                continue;
                            };
                            let n = update_word.execute(params![glyph, ayah_id, position])?;
                            if n == 0 {
                                unmatched += 1;
                            } else {
                                word_updates += n as u32;
                            }
                        }
                        "end" => {
                            let Some((ayah_id, position)) = resolve_ayah_position(w, &ayah_ids)
                            else {
                                continue;
                            };
                            let Some(prev) = position.checked_sub(1) else {
                                continue;
                            };
                            append_end.execute(params![glyph, ayah_id, prev])?;
                        }
                        "bismillah" => {
                            let Some(sura) = w.sura else { continue };
                            let Some(&word_id) = surah_basmala_word.get(&sura) else {
                                continue;
                            };
                            update_basmala.execute(params![glyph, word_id])?;
                            basmala_updates += 1;
                        }
                        "quarter" => {
                            // No `position` of its own: the ornament belongs to
                            // whichever word comes next on the line, and is
                            // resolved through that word's key.
                            let Some(next) =
                                line.words[i + 1..].iter().find(|n| n.word_type == "word")
                            else {
                                unmatched += 1;
                                continue;
                            };
                            let Some((ayah_id, position)) = resolve_ayah_position(next, &ayah_ids)
                            else {
                                unmatched += 1;
                                continue;
                            };
                            quarters.push((glyph, ayah_id, position));
                        }
                        _ => {} // "surah_header" — see doc comment above
                    }
                }
            }
        }
    }

    // Second pass: every word glyph is now written, so prepending a rub-el-hizb
    // ornament can't be clobbered by the word it belongs to.
    {
        let mut prepend_quarter = tx.prepare(
            "UPDATE page_line_word SET glyph_v4 = ?1 || ' ' || glyph_v4
             WHERE ayah_id = ?2 AND word_index = ?3 AND glyph_v4 IS NOT NULL",
        )?;
        for (glyph, ayah_id, position) in &quarters {
            let n = prepend_quarter.execute(params![glyph, ayah_id, position])?;
            if n == 0 {
                unmatched += 1;
            } else {
                quarter_updates += n as u32;
            }
        }
    }

    tx.commit().context("Committing QCF v4 glyph update")?;

    log::info!(
        "      Updated {word_updates} word glyphs, {basmala_updates} basmala glyphs, {quarter_updates} rub-el-hizb ornaments ({unmatched} v4 entries unmatched to any v2 row)"
    );

    let still_null: u32 = conn.query_row(
        "SELECT COUNT(*) FROM page_line_word plw
         JOIN page_line pl ON pl.id = plw.page_line_id
         WHERE plw.glyph_v4 IS NULL AND pl.line_type IN ('text', 'basmala')",
        [],
        |r| r.get(0),
    )?;
    if still_null > 0 {
        log::warn!(
            "      {still_null} page_line_word row(s) still have no glyph_v4 after the v4 pass"
        );
    }

    Ok(())
}

fn resolve_ayah_position(
    w: &WordV4Json,
    ayah_ids: &HashMap<(u32, u32), i64>,
) -> Option<(i64, u32)> {
    let verse_key = w.verse_key.as_deref()?;
    let position = w.position?;
    let (surah_id, ayah_number) = parse_verse_key(verse_key)?;
    let ayah_id = *ayah_ids.get(&(surah_id, ayah_number))?;
    Some((ayah_id, position))
}

fn parse_verse_key(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once(':')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}
