/// Tafsir import — fetches a commentary edition per Surah, normalises it to
/// plain text, and writes it into the app database.
///
/// Data source: spa5k/tafsir_api (MIT tooling; the underlying texts carry their
/// own terms, recorded per edition in `license` below and in
/// THIRD-PARTY-NOTICES.md). Each edition exposes one JSON file per Surah at
/// `tafsir/<slug>/<surah>.json`, an array of `{surah, ayah, text}`.
///
/// Which editions may appear here is a deliberate, narrow question — see
/// `EDITIONS`.
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::fetch;

const TAFSIR_RAW_BASE: &str = "https://raw.githubusercontent.com/spa5k/tafsir_api/main/tafsir";

/// A commentary edition this importer is willing to ship.
pub struct Edition {
    /// Stable id in the source repo, and the app's own id for the edition.
    pub slug: &'static str,
    pub title: &'static str,
    pub name_native: Option<&'static str>,
    pub author: &'static str,
    /// Set when this edition is a translation of the work rather than the
    /// original — the attribution belongs to both.
    pub translator: Option<&'static str>,
    pub language: &'static str,
    pub direction: &'static str,
    pub school: &'static str,
    pub creed: &'static str,
    pub source_url: &'static str,
    pub license: &'static str,
    pub version: &'static str,
}

/// The editions available to this app.
///
/// This list is the whole filter. A commentary's madhhab decides how it reads
/// the ayat al-ahkam and its creed decides how it reads the attribute verses,
/// and neither is visible in the text itself — so rather than offer everything
/// and label it, this reader only carries works of the Shafi'i school and
/// Ash'ari creed. Adding an edition outside that is a decision about the app's
/// purpose, not a data-sourcing convenience.
///
/// Deliberately absent, for the record: Ibn Kathir (Shafi'i in fiqh but Athari
/// in creed), al-Qurtubi (Maliki), al-Nasafi (Hanafi/Maturidi), al-Sa'di
/// (Hanbali/Salafi), al-Muyassar and al-Mukhtasar (Saudi committee), Fi Zilal
/// (Qutb), Tafhim (Mawdudi).
pub const EDITIONS: &[Edition] = &[Edition {
    slug: "tafsir-al-jalalayn",
    title: "Tafsīr al-Jalālayn",
    name_native: Some("تفسير الجلالين"),
    author: "Jalāl al-Dīn al-Maḥallī & Jalāl al-Dīn al-Suyūṭī",
    translator: Some("Feras Hamza"),
    language: "en",
    direction: "ltr",
    school: "shafii",
    creed: "ashari",
    source_url: "https://github.com/spa5k/tafsir_api",
    license: "© 2007 Royal Aal al-Bayt Institute for Islamic Thought — see THIRD-PARTY-NOTICES.md",
    version: "1.0",
}];

pub fn find_edition(slug: &str) -> Option<&'static Edition> {
    EDITIONS.iter().find(|e| e.slug == slug)
}

#[derive(Debug, Deserialize)]
struct EntryJson {
    surah: u32,
    ayah: u32,
    text: String,
}

/// One commentary entry, already mapped onto our own Ayah numbering.
pub struct Entry {
    pub surah: u32,
    pub ayah: u32,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Fetch
// ---------------------------------------------------------------------------

/// Fetch all 114 Surah files for an edition. Sequential on purpose: the Mushaf
/// import's 604-request run showed what a stalled connection to a CDN costs a
/// long fan-out, and 114 requests are quick enough not to need the risk.
pub fn fetch_edition(edition: &Edition) -> Result<Vec<Entry>> {
    let mut entries = Vec::with_capacity(6236);

    for surah in 1..=114u32 {
        let url = format!("{}/{}/{}.json", TAFSIR_RAW_BASE, edition.slug, surah);
        let body = fetch::get_pub(&url).with_context(|| format!("Fetching {url}"))?;
        entries.extend(parse_surah(&body, surah)?);

        if surah % 20 == 0 || surah == 114 {
            log::info!("      … {surah}/114 surahs");
        }
    }

    Ok(entries)
}

/// Load the same files from a local directory of `1.json … 114.json` instead
/// of fetching them, for a re-run without 114 requests.
pub fn load_edition(dir: &Path) -> Result<Vec<Entry>> {
    let mut entries = Vec::with_capacity(6236);

    for surah in 1..=114u32 {
        let path = dir.join(format!("{surah}.json"));
        let body = std::fs::read_to_string(&path)
            .with_context(|| format!("Reading {}", path.display()))?;
        entries.extend(parse_surah(&body, surah)?);
    }

    Ok(entries)
}

fn parse_surah(body: &str, surah: u32) -> Result<Vec<Entry>> {
    let raw: Vec<EntryJson> =
        serde_json::from_str(body).with_context(|| format!("Parsing surah {surah} JSON"))?;

    let mut out = Vec::with_capacity(raw.len());
    for e in raw {
        if e.surah != surah {
            bail!(
                "Surah {surah} file contains an entry for surah {} (ayah {})",
                e.surah,
                e.ayah
            );
        }
        let text = normalize(&e.text);
        // A source that carries the key but no commentary is not an error —
        // it is the same thing as the key being absent, which is ordinary in
        // this data (see the coverage check in `write_tafsir`).
        if text.is_empty() {
            continue;
        }
        out.push(Entry {
            surah: e.surah,
            ayah: e.ayah,
            text,
        });
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Normalisation
// ---------------------------------------------------------------------------

/// Reduce a source entry to plain text.
///
/// Markup is stripped rather than allow-listed. The editions carried here are
/// plain text already (checked across all 6,236 entries of al-Jalalayn), so an
/// allowlist would preserve nothing and would leave the renderer having to
/// trust the database. If an edition with meaningful markup is ever added,
/// this is the decision to revisit — and the renderer changes with it.
fn normalize(input: &str) -> String {
    let no_tags = strip_tags(input);
    let decoded = decode_entities(&no_tags);

    // Collapse horizontal whitespace, keep paragraph breaks, drop the rest.
    let mut out = String::with_capacity(decoded.len());
    let mut blank_run = 0;
    for line in decoded.lines() {
        let collapsed = collapse_spaces(line);
        if collapsed.is_empty() {
            blank_run += 1;
            continue;
        }
        if !out.is_empty() {
            out.push_str(if blank_run > 0 { "\n\n" } else { "\n" });
        }
        blank_run = 0;
        out.push_str(&collapsed);
    }

    out.trim().to_string()
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth = 0usize;
    for ch in input.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            // A tag boundary is also a word boundary: "a<br>b" must not become
            // "ab". The whitespace collapse above tidies up the extra space.
            _ if depth > 0 => {}
            _ => out.push(ch),
        }
        if ch == '>' && depth == 0 {
            out.push(' ');
        }
    }
    out
}

fn decode_entities(input: &str) -> String {
    // The five predefined XML entities plus the two that actually turn up in
    // this data. Numeric references are left alone: they don't appear, and a
    // half-implemented decoder is worse than none.
    input
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        // Last, so an escaped entity ("&amp;lt;") doesn't decode twice.
        .replace("&amp;", "&")
}

fn collapse_spaces(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_space = false;
    for ch in line.chars() {
        if ch.is_whitespace() {
            in_space = true;
            continue;
        }
        if in_space && !out.is_empty() {
            out.push(' ');
        }
        in_space = false;
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

/// Insert (or replace) an edition and all of its entries.
///
/// `bundled` marks the edition as shipping with the app, which is what stops
/// an upgrade from deleting it (see `copy_bundled_tafsir_from_seed` in the
/// app's connection.rs) — so it is only true for editions written into the
/// seed database that this repo commits.
pub fn write_tafsir(
    db_path: &Path,
    edition: &Edition,
    entries: &[Entry],
    bundled: bool,
) -> Result<()> {
    let conn =
        Connection::open(db_path).with_context(|| format!("Opening {}", db_path.display()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    ensure_migration_006(&conn)?;

    // (surah, ayah) -> ayah.id
    let ayah_ids: HashMap<(u32, u32), u32> = conn
        .prepare("SELECT surah_id, ayah_number, id FROM ayah")?
        .query_map([], |row| {
            Ok((
                (row.get::<_, u32>(0)?, row.get::<_, u32>(1)?),
                row.get::<_, u32>(2)?,
            ))
        })?
        .collect::<Result<_, _>>()?;

    let total_ayahs = ayah_ids.len();
    if total_ayahs == 0 {
        bail!(
            "No ayahs in {} — run the main import first",
            db_path.display()
        );
    }

    // Validate before writing anything.
    let mut seen = std::collections::HashSet::new();
    for e in entries {
        if !ayah_ids.contains_key(&(e.surah, e.ayah)) {
            bail!("Entry {}:{} is not a real ayah", e.surah, e.ayah);
        }
        if !seen.insert((e.surah, e.ayah)) {
            bail!("Duplicate entry for {}:{}", e.surah, e.ayah);
        }
    }

    let coverage = entries.len() as f64 / total_ayahs as f64;
    log::info!(
        "  → Coverage: {}/{} ayahs ({:.1}%)",
        entries.len(),
        total_ayahs,
        coverage * 100.0
    );
    // Gaps are normal and not corruption: al-Jalalayn glosses 6,010 of the
    // 6,236 ayahs, passing over verses that need no comment (47 of them in
    // al-Shu'ara alone, whose refrain repeats). A *large* gap is corruption,
    // hence the floor rather than an equality check.
    if coverage < 0.90 {
        bail!(
            "Only {:.1}% of ayahs have commentary — refusing to write what looks like a truncated source",
            coverage * 100.0
        );
    }

    let tafsir_id = upsert_edition(&conn, edition, bundled)?;

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM tafsir_ayah WHERE tafsir_id = ?1",
        params![tafsir_id],
    )?;
    {
        let mut stmt =
            tx.prepare("INSERT INTO tafsir_ayah (tafsir_id, ayah_id, text) VALUES (?1, ?2, ?3)")?;
        for e in entries {
            let ayah_id = ayah_ids[&(e.surah, e.ayah)];
            stmt.execute(params![tafsir_id, ayah_id, e.text])
                .with_context(|| format!("Inserting {}:{}", e.surah, e.ayah))?;
        }
    }
    tx.commit().context("Committing tafsir entries")?;

    // group_start/group_end stay null: this edition comments ayah by ayah.
    // A grouped edition fills them at import time so the panel can label the
    // run once instead of repeating the block.

    conn.execute_batch("INSERT INTO fts_tafsir(fts_tafsir) VALUES ('rebuild');")?;

    // This file is embedded in the binary with include_bytes!, so every page
    // of slack left by the delete-and-reinsert above is paid for in all seven
    // release installers.
    conn.execute_batch("VACUUM;")?;

    let rows: u32 = conn.query_row(
        "SELECT COUNT(*) FROM tafsir_ayah WHERE tafsir_id = ?1",
        params![tafsir_id],
        |r| r.get(0),
    )?;
    let bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(text)), 0) FROM tafsir_ayah WHERE tafsir_id = ?1",
        params![tafsir_id],
        |r| r.get(0),
    )?;
    log::info!(
        "  → Wrote {} rows ({:.2} MB of text) as tafsir id {}",
        rows,
        bytes as f64 / 1_048_576.0,
        tafsir_id
    );

    Ok(())
}

fn upsert_edition(conn: &Connection, edition: &Edition, bundled: bool) -> Result<u32> {
    conn.execute(
        "INSERT INTO tafsir
            (language, author, title, version, is_bundled, slug, translator,
             name_native, direction, school, creed, source_url, license, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0)
         ON CONFLICT(slug) DO UPDATE SET
            language    = excluded.language,
            author      = excluded.author,
            title       = excluded.title,
            version     = excluded.version,
            is_bundled  = excluded.is_bundled,
            translator  = excluded.translator,
            name_native = excluded.name_native,
            direction   = excluded.direction,
            school      = excluded.school,
            creed       = excluded.creed,
            source_url  = excluded.source_url,
            license     = excluded.license",
        params![
            edition.language,
            edition.author,
            edition.title,
            edition.version,
            bundled as i64,
            edition.slug,
            edition.translator,
            edition.name_native,
            edition.direction,
            edition.school,
            edition.creed,
            edition.source_url,
            edition.license,
        ],
    )?;

    let id: u32 = conn.query_row(
        "SELECT id FROM tafsir WHERE slug = ?1",
        params![edition.slug],
        |r| r.get(0),
    )?;
    Ok(id)
}

/// Apply migration 006 if this database predates it. The committed seed is
/// still at the schema version it was built with, so the importer has to be
/// able to bring it forward on its own rather than assuming a fresh
/// `schema.sql` run.
fn ensure_migration_006(conn: &Connection) -> Result<()> {
    let has_slug: Option<String> = conn
        .query_row(
            "SELECT name FROM pragma_table_info('tafsir') WHERE name = 'slug'",
            [],
            |r| r.get(0),
        )
        .optional()?;

    if has_slug.is_some() {
        return Ok(());
    }

    log::info!("  → Applying migration 006 to the database");
    conn.execute_batch(include_str!("../../database/migrations/006_tafsir.sql"))
        .context("Applying migration 006")?;
    Ok(())
}

/// Resolve `--tafsir-dir <path>` into a directory of `1.json … 114.json`.
pub fn local_dir_arg(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == "--tafsir-dir")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markup_without_gluing_words_together() {
        assert_eq!(normalize("a<br>b"), "a b");
        assert_eq!(normalize("<p>Hello  <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn decodes_entities_once() {
        assert_eq!(normalize("Moses&apos; people"), "Moses' people");
        assert_eq!(normalize("&amp;lt;"), "&lt;");
    }

    #[test]
    fn keeps_paragraph_breaks_and_trims() {
        assert_eq!(normalize("  one\n\n\n  two  \n"), "one\n\ntwo");
    }
}
