/// Mushaf page layout — imports the line-by-line Madani print layout so a
/// page can be rendered with the same line breaks and word placement as the
/// printed Mushaf.
///
/// Data source: MohamadHajjRabee/quran-qcf4 (MIT JSON), 604 page files.
/// Glyphs are QCF v4 (King Fahd Complex, Uthman Taha calligraphy); the
/// matching font files are vendored separately (see
/// scripts/vendor-mushaf-fonts-v4.sh).
///
/// This used to be a two-source import: zonetecde/mushaf-layout supplied the
/// line structure and v2 glyphs, and a second pass *attached* v4 glyphs to the
/// rows it had already created, matching on `(ayah_id, word_index)`. That join
/// was wrong in both directions:
///
///   * The two sources don't segment words identically. The v2 layout packs
///     two v4 tokens into one row in 19 Ayahs — every sajdah `۩`, the three
///     `بَعْدَ مَا`, and 37:130's `إِلْ يَاسِينَ` — so the surplus v4 glyph and
///     the Ayah-end marker after it matched no row and were silently dropped.
///     37:130 rendered as `سَلَامٌ عَلَىٰ إِلْ` with no `يَاسِينَ` and no ﴿١٣٠﴾.
///   * The two sources don't break lines identically either. 4785 of 8820 text
///     lines placed their words differently, so v4 glyphs — whose advances are
///     cut for v4's own line breaks — were being laid out on v2's.
///
/// Since nothing reads v2 glyphs any more, the fix is to stop consulting v2 at
/// all and build the whole layout from the source the fonts belong to. Per-word
/// Uthmani text, which is the one thing v4 doesn't carry in the right
/// orthography (its `text` field is a simplified script), comes from splitting
/// our own already-validated `ayah.uthmani_text` — whose token count matches v4's
/// word count for all 6236 Ayahs once the sajdah ornament, which is a print
/// mark rather than a word, is set aside.
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const V4_RAW_BASE: &str = "https://raw.githubusercontent.com/MohamadHajjRabee/quran-qcf4/main";

/// The `text` a v4 word entry carries when it is really the sajdah ornament
/// `۩` rather than a word — the only entry typed `"word"` that has no Uthmani
/// token of its own. It always sits at the end of its Ayah, on all 15 sajdah
/// Ayahs and nowhere else (both asserted in `validate_layout`).
const SAJDAH_ENTRY_TEXT: &str = "#1969";
const SAJDAH_SIGN: char = '۩';
const QUARTER_SIGN: char = '۞';

#[derive(Debug, Deserialize)]
pub struct PageV4Json {
    page: u32,
    lines: Vec<LineV4Json>,
}

#[derive(Debug, Deserialize)]
struct LineV4Json {
    line: u32,
    words: Vec<WordV4Json>,
}

#[derive(Debug, Deserialize)]
struct WordV4Json {
    code: u32,
    #[serde(rename = "type")]
    word_type: String, // "word" | "end" | "surah_header" | "bismillah" | "quarter"
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    verse_key: Option<String>,
    #[serde(default)]
    position: Option<u32>, // 1-based within the Ayah, counting the sajdah ornament
    #[serde(default)]
    sura: Option<u32>,
}

/// Load all 604 QCF v4 page-layout files from a local directory of
/// `001.json` … `604.json` — the layout the upstream repo and its npm package
/// (`quran-qcf4`) both use, so extracting either one gives a usable directory.
///
/// Preferred over `fetch_all_pages` when you have the files: one 604-file
/// read beats 604 sequential HTTPS requests, and it makes the import repeatable
/// offline rather than dependent on a CDN staying reachable for ten minutes.
pub fn load_all_pages(dir: &Path) -> Result<Vec<PageV4Json>> {
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
pub fn fetch_all_pages() -> Result<Vec<PageV4Json>> {
    let mut pages = Vec::with_capacity(604);

    for n in 1..=604u32 {
        let url = format!("{V4_RAW_BASE}/pages/{n:03}.json");
        let body = crate::fetch::get_pub(&url)
            .with_context(|| format!("Fetching QCF v4 layout page {n}"))?;
        let page: PageV4Json = serde_json::from_str(&body)
            .with_context(|| format!("Parsing QCF v4 layout page {n}"))?;
        pages.push(page);

        if n % 50 == 0 {
            log::info!("      … layout {n}/604");
        }
    }

    Ok(pages)
}

/// One `page_line_word` row under construction. Ayah-end markers, the sajdah
/// ornament and the rub-el-hizb `۞` are not words: each is folded into a
/// neighbouring word's row, so a row stays one *word* and a line keeps the word
/// count the printed line justifies to.
struct WordRow {
    ayah_id: Option<i64>,
    word_index: Option<u32>,
    uthmani: String,
    glyph: String,
}

/// Rebuild `page_line` / `page_line_word` from the v4 layout, replacing
/// whatever was there. Runs against a database that already has Surahs and
/// Ayahs — it needs `ayah.uthmani_text` for per-word text, and `surah.name_ar`
/// for the header banners.
///
/// Safe to re-run: it clears the two tables first, so a rebuild against an
/// already-imported database needs no re-fetch of anything else.
pub fn write_mushaf_layout(db_path: &Path, pages: &[PageV4Json]) -> Result<()> {
    let conn = Connection::open(db_path).context("Opening database for mushaf layout")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // (surah_id, ayah_number) -> ayah.id, and ayah.id -> its Uthmani words.
    let mut ayah_ids: HashMap<(u32, u32), i64> = HashMap::new();
    let mut ayah_words: HashMap<i64, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, surah_id, ayah_number, uthmani_text FROM ayah")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        for row in rows {
            let (id, surah_id, ayah_number, text) = row?;
            ayah_ids.insert((surah_id, ayah_number), id);
            ayah_words.insert(id, text.split_whitespace().map(str::to_string).collect());
        }
    }

    let mut surah_names: HashMap<u32, String> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, name_ar FROM surah")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (id, name) = row?;
            surah_names.insert(id, name);
        }
    }

    // Surah 1's Ayah 1 *is* the Basmala, so it doubles as the text of every
    // other Surah's Basmala header line.
    let basmala_text: String = conn.query_row(
        "SELECT uthmani_text FROM ayah WHERE surah_id = 1 AND ayah_number = 1",
        [],
        |r| r.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;

    // page_line_word is ON DELETE CASCADE, and foreign_keys is on above.
    let cleared = tx.execute("DELETE FROM page_line", [])?;
    if cleared > 0 {
        log::info!("      Cleared {cleared} existing page lines");
    }

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
             (page_line_id, position, ayah_id, word_index, uthmani_text, glyph_v4)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for page in pages {
            for line in &page.lines {
                let where_ = || format!("page {} line {}", page.page, line.line);
                let Some(first) = line.words.first() else {
                    bail!("{}: empty line", where_());
                };

                let rows: Vec<WordRow> = match first.word_type.as_str() {
                    // The banner is live-rendered from `surah.name_ar` rather
                    // than from the page's own header glyph (see
                    // SurahHeader.svelte), so the line carries the name as text
                    // and no word rows at all.
                    "surah_header" => {
                        let surah_id = first
                            .sura
                            .with_context(|| format!("{}: surah_header without `sura`", where_()))?;
                        let name = surah_names
                            .get(&surah_id)
                            .with_context(|| format!("{}: unknown Surah {surah_id}", where_()))?;
                        line_stmt.execute(params![
                            page.page,
                            line.line,
                            "surah_header",
                            surah_id,
                            Option::<i64>::None,
                            Option::<i64>::None,
                            name,
                        ])?;
                        line_count += 1;
                        continue;
                    }

                    // One whole-phrase glyph, carrying no Ayah of its own —
                    // the Basmala is a header, not Ayah 1 of the Surah.
                    "bismillah" => {
                        let ch = glyph_of(first, &where_)?;
                        vec![WordRow {
                            ayah_id: None,
                            word_index: None,
                            uthmani: basmala_text.clone(),
                            glyph: ch.to_string(),
                        }]
                    }

                    _ => build_text_line(line, &ayah_ids, &ayah_words, &where_)?,
                };

                let line_type = if first.word_type == "bismillah" {
                    "basmala"
                } else {
                    "text"
                };
                let first_ayah_id = rows.first().and_then(|r| r.ayah_id);
                let last_ayah_id = rows.last().and_then(|r| r.ayah_id);

                line_stmt.execute(params![
                    page.page,
                    line.line,
                    line_type,
                    Option::<u32>::None,
                    first_ayah_id,
                    last_ayah_id,
                    Option::<String>::None,
                ])?;
                let page_line_id = tx.last_insert_rowid();
                line_count += 1;

                for (position, row) in rows.iter().enumerate() {
                    word_stmt.execute(params![
                        page_line_id,
                        position as i64,
                        row.ayah_id,
                        row.word_index,
                        row.uthmani,
                        row.glyph,
                    ])?;
                    word_count += 1;
                }
            }
        }
    }

    validate_layout(&tx)?;

    tx.commit().context("Committing mushaf layout transaction")?;

    log::info!("      Inserted {line_count} page lines, {word_count} words");

    Ok(())
}

/// Turn one v4 text line's entries into the word rows it renders as.
///
/// Only `"word"` entries become rows. The other three fold into a neighbour:
///
///   * `"end"` (the ﴿٢٧﴾ Ayah marker) appends onto the word before it. The
///     source never starts a line with one, and never separates one from its
///     own Ayah's last word, so "the word before it" is always the right word
///     and always on this line (asserted below).
///   * the sajdah `۩` ornament — a `"word"` entry whose `text` is
///     `SAJDAH_ENTRY_TEXT` — likewise appends onto the word before it, and
///     always sits at the end of its Ayah, before that Ayah's end marker.
///   * `"quarter"` (the rub-el-hizb `۞`) *prepends* onto the word after it,
///     the mirror image; it can open a line but never closes one.
fn build_text_line(
    line: &LineV4Json,
    ayah_ids: &HashMap<(u32, u32), i64>,
    ayah_words: &HashMap<i64, Vec<String>>,
    where_: &impl Fn() -> String,
) -> Result<Vec<WordRow>> {
    let mut rows: Vec<WordRow> = Vec::with_capacity(line.words.len());
    let mut pending_quarter: Option<char> = None;

    for w in &line.words {
        let ch = glyph_of(w, where_)?;

        match w.word_type.as_str() {
            "quarter" => {
                if pending_quarter.replace(ch).is_some() {
                    bail!("{}: two rub-el-hizb marks with no word between", where_());
                }
            }

            "end" => {
                let (_, ayah_number) = verse_key_of(w, where_)?;
                let row = rows.last_mut().with_context(|| {
                    format!("{}: Ayah end marker with no word before it", where_())
                })?;
                row.glyph.push(' ');
                row.glyph.push(ch);
                row.uthmani.push(' ');
                row.uthmani.push_str(&arabic_indic(ayah_number));
            }

            "word" if w.text.as_deref() == Some(SAJDAH_ENTRY_TEXT) => {
                let row = rows.last_mut().with_context(|| {
                    format!("{}: sajdah ornament with no word before it", where_())
                })?;
                row.glyph.push(' ');
                row.glyph.push(ch);
                row.uthmani.push(' ');
                row.uthmani.push(SAJDAH_SIGN);
            }

            "word" => {
                let (surah_id, ayah_number) = verse_key_of(w, where_)?;
                let position = w
                    .position
                    .with_context(|| format!("{}: word without `position`", where_()))?;
                let ayah_id = *ayah_ids.get(&(surah_id, ayah_number)).with_context(|| {
                    format!("{}: unknown Ayah {surah_id}:{ayah_number}", where_())
                })?;
                // The sajdah ornament is the only non-word `"word"` entry and
                // always comes last in its Ayah, so every real word's
                // `position` indexes the Uthmani text directly.
                let uthmani = ayah_words
                    .get(&ayah_id)
                    .and_then(|ws| ws.get(position as usize - 1))
                    .with_context(|| {
                        format!(
                            "{}: {surah_id}:{ayah_number} has no Uthmani word {position}",
                            where_()
                        )
                    })?
                    .clone();

                let mut row = WordRow {
                    ayah_id: Some(ayah_id),
                    word_index: Some(position),
                    uthmani,
                    glyph: ch.to_string(),
                };
                if let Some(q) = pending_quarter.take() {
                    row.glyph.insert(0, ' ');
                    row.glyph.insert(0, q);
                    row.uthmani.insert(0, ' ');
                    row.uthmani.insert(0, QUARTER_SIGN);
                }
                rows.push(row);
            }

            other => bail!("{}: unexpected entry type `{other}` on a text line", where_()),
        }
    }

    if pending_quarter.is_some() {
        bail!("{}: line ends on a rub-el-hizb mark", where_());
    }
    if rows.is_empty() {
        bail!("{}: text line with no words", where_());
    }

    Ok(rows)
}

fn glyph_of(w: &WordV4Json, where_: &impl Fn() -> String) -> Result<char> {
    char::from_u32(w.code)
        .with_context(|| format!("{}: `code` {} is not a character", where_(), w.code))
}

fn verse_key_of(w: &WordV4Json, where_: &impl Fn() -> String) -> Result<(u32, u32)> {
    let key = w
        .verse_key
        .as_deref()
        .with_context(|| format!("{}: entry without `verse_key`", where_()))?;
    let (a, b) = key
        .split_once(':')
        .with_context(|| format!("{}: malformed verse_key `{key}`", where_()))?;
    Ok((a.parse()?, b.parse()?))
}

/// `130` -> `١٣٠`. The Ayah number as it reads inside the end marker, so a
/// row's `uthmani_text` says what its glyphs draw.
fn arabic_indic(n: u32) -> String {
    n.to_string()
        .chars()
        .map(|c| char::from_u32(0x0660 + c.to_digit(10).unwrap()).unwrap())
        .collect()
}

/// Everything the old two-source import got wrong silently, asserted loudly.
///
/// The previous pass counted its unmatched rows into a log line and carried on,
/// which is how 19 Ayahs shipped missing their last word and their end marker.
/// These run inside the import transaction, so a failure rolls the layout back
/// rather than committing a bad Mushaf.
fn validate_layout(tx: &rusqlite::Transaction) -> Result<()> {
    let one = |sql: &str| -> Result<i64> { Ok(tx.query_row(sql, [], |r| r.get(0))?) };

    let pages = one("SELECT COUNT(DISTINCT page) FROM page_line")?;
    if pages != 604 {
        bail!("Expected 604 Mushaf pages, got {pages}");
    }

    let headers = one("SELECT COUNT(*) FROM page_line WHERE line_type = 'surah_header'")?;
    if headers != 114 {
        bail!("Expected 114 surah_header lines, got {headers}");
    }

    // Every Surah is announced exactly once, on the page its first Ayah is on.
    let misplaced = one(
        "SELECT COUNT(*) FROM surah s
         JOIN ayah a ON a.surah_id = s.id AND a.ayah_number = 1
         WHERE NOT EXISTS (
           SELECT 1 FROM page_line pl
           WHERE pl.line_type = 'surah_header' AND pl.surah_id = s.id AND pl.page = a.page
         )",
    )?;
    if misplaced != 0 {
        bail!("{misplaced} Surah(s) have no header on their opening page");
    }

    // Every Surah but Al-Fatihah (whose Basmala is Ayah 1) and At-Tawbah
    // (which has none) opens with a Basmala line.
    let basmalas = one("SELECT COUNT(*) FROM page_line WHERE line_type = 'basmala'")?;
    if basmalas != 112 {
        bail!("Expected 112 basmala lines, got {basmalas}");
    }
    let missing_basmala = one(
        "SELECT COUNT(*) FROM surah s
         JOIN ayah a ON a.surah_id = s.id AND a.ayah_number = 1
         WHERE s.id NOT IN (1, 9) AND NOT EXISTS (
           SELECT 1 FROM page_line pl WHERE pl.page = a.page AND pl.line_type = 'basmala'
         )",
    )?;
    if missing_basmala != 0 {
        bail!("{missing_basmala} Surah(s) open with no Basmala line");
    }

    // Nothing renders as an empty box.
    let no_glyph = one(
        "SELECT COUNT(*) FROM page_line_word
         WHERE glyph_v4 IS NULL OR glyph_v4 = ''",
    )?;
    if no_glyph != 0 {
        bail!("{no_glyph} word row(s) have no glyph");
    }

    // Every Ayah is laid out, and laid out whole: one row per Uthmani word.
    let laid_out = one("SELECT COUNT(DISTINCT ayah_id) FROM page_line_word WHERE ayah_id IS NOT NULL")?;
    if laid_out != 6236 {
        bail!("Expected all 6236 Ayahs in the layout, got {laid_out}");
    }
    let short = one(
        "SELECT COUNT(*) FROM (
           SELECT a.id FROM ayah a
           JOIN page_line_word w ON w.ayah_id = a.id
           GROUP BY a.id
           HAVING COUNT(*) <> LENGTH(TRIM(a.uthmani_text)) - LENGTH(REPLACE(TRIM(a.uthmani_text), ' ', '')) + 1
         )",
    )?;
    if short != 0 {
        bail!("{short} Ayah(s) have a word-row count that disagrees with their Uthmani text");
    }

    // This is the bug that started all of it: an Ayah whose last word never got
    // its ﴿n﴾ marker appended. Every Ayah's highest-indexed word must carry a
    // second glyph.
    let no_marker = one(
        "SELECT COUNT(*) FROM (
           SELECT w.ayah_id FROM page_line_word w
           JOIN (SELECT ayah_id, MAX(word_index) AS mw FROM page_line_word
                 WHERE ayah_id IS NOT NULL GROUP BY ayah_id) last
             ON last.ayah_id = w.ayah_id AND last.mw = w.word_index
           WHERE INSTR(w.glyph_v4, ' ') = 0
         )",
    )?;
    if no_marker != 0 {
        bail!("{no_marker} Ayah(s) end without an Ayah-marker glyph");
    }

    // The 15 sajdah ornaments land on the 15 sajdah Ayahs and nowhere else.
    let sajdah_rows = one(
        "SELECT COUNT(*) FROM page_line_word w JOIN ayah a ON a.id = w.ayah_id
         WHERE INSTR(w.uthmani_text, '۩') > 0 AND a.sajdah = 1",
    )?;
    let sajdah_stray = one(
        "SELECT COUNT(*) FROM page_line_word w JOIN ayah a ON a.id = w.ayah_id
         WHERE INSTR(w.uthmani_text, '۩') > 0 AND a.sajdah = 0",
    )?;
    if sajdah_rows != 15 || sajdah_stray != 0 {
        bail!("Expected 15 sajdah ornaments on sajdah Ayahs, got {sajdah_rows} (+{sajdah_stray} stray)");
    }

    log::info!("      ✓ 604 pages, 114 headers, 112 basmalas, 6236 Ayahs, all end markers present");
    Ok(())
}
