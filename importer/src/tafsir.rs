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
use rusqlite::{params, Connection};
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
    /// Position in the picker. Also decides the fallback edition when the user
    /// has never chosen one, since the frontend takes the first in this order.
    pub sort_order: i64,
    /// Whether this edition comments on runs of verses rather than single
    /// Ayahs. Grouped sources repeat the whole block under every Ayah of the
    /// run, so the importer has to detect the runs and record them — see
    /// `group_entries`. False for per-Ayah editions like al-Jalalayn.
    pub grouped: bool,
}

/// The editions this app knows how to import.
///
/// Each entry carries its own `school` and `creed` so the picker can say whose
/// reading it is offering rather than presenting every commentary as
/// interchangeable. That is a label on the edition, not a gate on the list:
/// what belongs here is a question about the works themselves, decided edition
/// by edition, and this list is not the place to argue it.
///
/// None of them ship with the app. The seed database this repo commits carries
/// no commentary at all, so every edition here reaches a reader the same way:
/// built into a content pack by `emit_pack`, published, and downloaded on
/// request. There is deliberately no path from this list into the seed —
/// writing one there would put it in the binary, since the seed is embedded
/// with `include_bytes!`.
pub const EDITIONS: &[Edition] = &[
    Edition {
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
        license:
            "© 2007 Royal Aal al-Bayt Institute for Islamic Thought — see THIRD-PARTY-NOTICES.md",
        version: "1.0",
        // Listed first, which also makes it the fallback when no edition has
        // been chosen (see `active` in the frontend store). Keeping the English
        // ahead of the Arabic original is not a claim about the texts: it is
        // what stops every existing install that never picked an edition from
        // silently switching to Arabic on upgrade. The picker makes the
        // original one click away.
        sort_order: 0,
        grouped: false,
    },
    Edition {
        // The Arabic original. Verified as the only Arabic Jalālayn in the
        // source (QUL id 523); the other three editions there are two English
        // and one Indonesian, so the punctuation trap that separates the two
        // English editions has no equivalent on this side.
        slug: "ar-tafsir-al-jalalayn",
        title: "Tafsīr al-Jalālayn",
        name_native: Some("تفسير الجلالين"),
        author: "Jalāl al-Dīn al-Maḥallī & Jalāl al-Dīn al-Suyūṭī",
        // The original, not a translation. The panel renders "tr. X" off this
        // field, so it has to stay null or it will credit a translator to a
        // text that has none.
        translator: None,
        language: "ar",
        direction: "rtl",
        school: "shafii",
        creed: "ashari",
        source_url: "https://github.com/spa5k/tafsir_api",
        // The English entry's copyright belongs to Hamza's translation and does
        // not reach back to the work itself: al-Maḥallī died in 864 AH and
        // al-Suyūṭī in 911 AH, so the Arabic text is long out of copyright. What
        // is being credited here is the digital edition, not the composition.
        license: "Public domain (composed 864–911 AH) — digital edition via spa5k/tafsir_api, sourced from qul.tarteel.ai",
        version: "1.0",
        sort_order: 10,
        grouped: false,
    },
    Edition {
        // The Darussalam abridgement, and the slug's "tafisr" is the source's
        // own typo — not a transcription error here. Verified against the
        // repo's directory listing; correcting it fetches nothing.
        //
        // Not the same book as the Arabic below it, which is why the two are
        // separate entries rather than one edition in two languages: the
        // abridgement drops most of the isnads and much of the linguistic
        // discussion, and carries an editorial apparatus of its own.
        slug: "en-tafisr-ibn-kathir",
        title: "Tafsīr Ibn Kathīr (abridged)",
        name_native: Some("تفسير ابن كثير"),
        author: "Ismāʿīl ibn ʿUmar ibn Kathīr",
        translator: Some("Abridged under Ṣafī al-Raḥmān al-Mubārakfūrī"),
        language: "en",
        direction: "ltr",
        school: "shafii",
        creed: "athari",
        source_url: "https://github.com/spa5k/tafsir_api",
        // Unsettled, and said so rather than left blank: the abridgement is a
        // modern Darussalam publication with a live copyright, unlike the
        // Arabic original below. THIRD-PARTY-NOTICES.md carries the same
        // caveat. It is also the reason this stays a downloadable pack and out
        // of the installer until the terms are established.
        license: "Abridgement © Darussalam — redistribution terms UNVERIFIED, see THIRD-PARTY-NOTICES.md",
        version: "1.0",
        sort_order: 20,
        grouped: true,
    },
    Edition {
        // The Arabic original, unabridged.
        slug: "ar-tafsir-ibn-kathir",
        title: "Tafsīr Ibn Kathīr",
        name_native: Some("تفسير ابن كثير"),
        author: "Ismāʿīl ibn ʿUmar ibn Kathīr",
        // The original, not a translation — same reason as the Arabic
        // al-Jalalayn above: the panel renders "tr. X" off this field.
        translator: None,
        language: "ar",
        direction: "rtl",
        school: "shafii",
        creed: "athari",
        source_url: "https://github.com/spa5k/tafsir_api",
        license: "Public domain (d. 774 AH) — digital edition via spa5k/tafsir_api",
        version: "1.0",
        sort_order: 30,
        grouped: true,
    },
];

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
        // this data (see the coverage check in `prepare`).
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
// Grouping
// ---------------------------------------------------------------------------

/// One run of consecutive Ayahs that share a single block of commentary.
pub struct Group {
    pub surah: u32,
    pub start_ayah: u32,
    pub end_ayah: u32,
    pub text: String,
}

impl Group {
    /// A run of one is not a run: the group columns stay null for it, so a
    /// grouped edition's isolated verses have exactly the shape a per-Ayah
    /// edition's do.
    fn is_run(&self) -> bool {
        self.end_ayah > self.start_ayah
    }
}

/// Collapse a grouped edition's entries into the runs the source actually
/// meant.
///
/// The source has no notion of a run: an edition that comments on 2:1-5 as one
/// block simply repeats that whole block under each of the five Ayahs. So the
/// runs have to be recovered, and byte-identical text on consecutive Ayahs of
/// one Surah is what recovers them. Ibn Kathir's blocks average 3.3 Ayahs and
/// reach 20, which is why this matters: stored naively the two editions cost
/// 126.6 MB where the runs cost 34.5 MB.
///
/// Identity is deliberately exact rather than fuzzy. Two adjacent Ayahs whose
/// commentary differs by so much as a space are two blocks, and treating them
/// as one would put text under a verse it was not written for.
///
/// `entries` must be sorted by (surah, ayah); `prepare` sorts before calling.
pub fn group_entries(entries: Vec<Entry>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();

    for e in entries {
        match groups.last_mut() {
            // Same Surah, the very next Ayah, and the same block of text.
            Some(g) if g.surah == e.surah && g.end_ayah + 1 == e.ayah && g.text == e.text => {
                g.end_ayah = e.ayah;
            }
            _ => groups.push(Group {
                surah: e.surah,
                start_ayah: e.ayah,
                end_ayah: e.ayah,
                text: e.text,
            }),
        }
    }

    groups
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

/// One row of `tafsir_ayah` as it will be stored: the Ayah it belongs to, the
/// block of commentary (empty for every Ayah of a run but its first), and the
/// bounds of the run it belongs to.
type Row = (u32, String, Option<u32>, Option<u32>);

/// One edition laid out as rows, with the counts worth logging.
struct Prepared {
    rows: Vec<Row>,
    covered: usize,
    blocks: usize,
    runs: usize,
    longest: u32,
}

/// `(surah, ayah) -> ayah.id` for the whole Mushaf.
fn load_ayah_ids(conn: &Connection) -> Result<HashMap<(u32, u32), u32>> {
    let ids: HashMap<(u32, u32), u32> = conn
        .prepare("SELECT surah_id, ayah_number, id FROM ayah")?
        .query_map([], |row| {
            Ok((
                (row.get::<_, u32>(0)?, row.get::<_, u32>(1)?),
                row.get::<_, u32>(2)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Validate an edition's entries against the Ayah table and lay them out as
/// `tafsir_ayah` rows.
///
/// Shared by the seed writer and the pack writer, which differ only in where
/// the rows end up — so an edition cannot be stored one way in the installer
/// and another way in a download.
fn prepare(
    ayah_ids: &HashMap<(u32, u32), u32>,
    edition: &Edition,
    entries: Vec<Entry>,
) -> Result<Prepared> {
    let total_ayahs = ayah_ids.len();

    let mut seen = std::collections::HashSet::new();
    for e in &entries {
        if !ayah_ids.contains_key(&(e.surah, e.ayah)) {
            bail!("Entry {}:{} is not a real ayah", e.surah, e.ayah);
        }
        if !seen.insert((e.surah, e.ayah)) {
            bail!("Duplicate entry for {}:{}", e.surah, e.ayah);
        }
    }

    let covered = entries.len();
    let coverage = covered as f64 / total_ayahs as f64;
    log::info!(
        "  → Coverage: {}/{} ayahs ({:.1}%)",
        covered,
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

    // Sorted before grouping, because a run is defined by adjacency: the
    // source files arrive one Surah at a time and in order, but that is the
    // source's habit rather than a guarantee this can rest on.
    let mut entries = entries;
    entries.sort_by_key(|e| (e.surah, e.ayah));

    let groups = if edition.grouped {
        group_entries(entries)
    } else {
        // Every entry is its own block. Going through the same path rather
        // than a second one keeps a per-Ayah edition's rows identical to what
        // they were before grouping existed.
        entries
            .into_iter()
            .map(|e| Group {
                surah: e.surah,
                start_ayah: e.ayah,
                end_ayah: e.ayah,
                text: e.text,
            })
            .collect()
    };

    let mut rows: Vec<Row> = Vec::with_capacity(covered);
    for g in &groups {
        let start_id = ayah_ids[&(g.surah, g.start_ayah)];
        let end_id = ayah_ids[&(g.surah, g.end_ayah)];

        // The run is stored as an id range, so a Surah's Ayah ids have to run
        // consecutively for the range to mean what it says. They do — `ayah`
        // is imported in Mushaf order and never renumbered — but a silent
        // violation here would hand verses commentary written for their
        // neighbours, so it is checked rather than assumed.
        if end_id - start_id != g.end_ayah - g.start_ayah {
            bail!(
                "Ayah ids for {}:{}-{} are not consecutive ({start_id}..{end_id}) — the run cannot be stored as a range",
                g.surah,
                g.start_ayah,
                g.end_ayah
            );
        }

        let (group_start, group_end) = if g.is_run() {
            (Some(start_id), Some(end_id))
        } else {
            (None, None)
        };

        // The block goes on the run's first Ayah and nowhere else. The rest of
        // the run keeps its row — so a lookup by (tafsir_id, ayah_id) is still
        // the point query it always was — but carries an empty text and points
        // at the row holding the real one. `get_tafsir_for_ayah` in the app
        // follows that pointer in the same statement.
        //
        // Storing the block under every Ayah of its run instead costs 3.7x the
        // space on Ibn Kathir (126.6 MB against 34.5 MB across the Arabic and
        // English editions) and makes every full text search return the same
        // block once per verse of the run.
        for ayah in g.start_ayah..=g.end_ayah {
            let ayah_id = ayah_ids[&(g.surah, ayah)];
            let text = if ayah == g.start_ayah {
                g.text.clone()
            } else {
                String::new()
            };
            rows.push((ayah_id, text, group_start, group_end));
        }
    }

    let runs = groups.iter().filter(|g| g.is_run()).count();
    let longest = groups
        .iter()
        .map(|g| g.end_ayah - g.start_ayah + 1)
        .max()
        .unwrap_or(0);

    if edition.grouped {
        log::info!(
            "  → {} blocks, {} of them runs, longest {} ayahs",
            groups.len(),
            runs,
            longest
        );
    }

    Ok(Prepared {
        rows,
        covered,
        blocks: groups.len(),
        runs,
        longest,
    })
}

// ---------------------------------------------------------------------------
// Content packs
// ---------------------------------------------------------------------------

/// Bumped when the pack layout changes in a way an older app cannot read. The
/// installer refuses a pack whose format it does not know rather than guessing
/// at the columns.
const PACK_FORMAT: u32 = 1;

/// Write one edition as a standalone content pack — a SQLite file the app
/// downloads, verifies and ATTACHes, copying the rows straight across.
///
/// Deliberately the same shape as the two tables it feeds, so the install is
/// an `INSERT ... SELECT` and not a parser. The one column that is *not*
/// carried over is `tafsir.id`: the installer allocates that itself, out of
/// the range the seed uses, because a pack that chose its own id would collide
/// with a bundled edition on the primary key.
///
/// `ayah_count` is recorded so the installer can refuse a pack built against a
/// different Ayah numbering. Every row here is keyed by `ayah.id`, which only
/// means anything against the Mushaf it was built from.
pub fn emit_pack(db_path: &Path, edition: &Edition, entries: Vec<Entry>, out: &Path) -> Result<()> {
    let src =
        Connection::open(db_path).with_context(|| format!("Opening {}", db_path.display()))?;
    let ayah_ids = load_ayah_ids(&src)?;
    if ayah_ids.is_empty() {
        bail!(
            "No ayahs in {} — run the main import first",
            db_path.display()
        );
    }

    let prepared = prepare(&ayah_ids, edition, entries)?;

    if out.exists() {
        std::fs::remove_file(out)
            .with_context(|| format!("Replacing existing pack {}", out.display()))?;
    }
    let pack = Connection::open(out).with_context(|| format!("Creating {}", out.display()))?;

    pack.execute_batch(
        "CREATE TABLE pack_meta (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );
         -- Same columns as the app's `tafsir`, minus `id`.
         CREATE TABLE tafsir (
             slug        TEXT NOT NULL,
             language    TEXT NOT NULL,
             author      TEXT NOT NULL,
             title       TEXT NOT NULL,
             version     TEXT NOT NULL,
             translator  TEXT,
             name_native TEXT,
             direction   TEXT NOT NULL,
             school      TEXT,
             creed       TEXT,
             source_url  TEXT,
             license     TEXT,
             sort_order  INTEGER NOT NULL
         );
         -- Same columns as the app's `tafsir_ayah`, minus `tafsir_id`.
         CREATE TABLE tafsir_ayah (
             ayah_id             INTEGER NOT NULL PRIMARY KEY,
             text                TEXT NOT NULL,
             group_start_ayah_id INTEGER,
             group_end_ayah_id   INTEGER
         );",
    )?;

    let tx = pack.unchecked_transaction()?;
    {
        let mut meta = tx.prepare("INSERT INTO pack_meta (key, value) VALUES (?1, ?2)")?;
        for (k, v) in [
            ("format", PACK_FORMAT.to_string()),
            ("slug", edition.slug.to_string()),
            ("version", edition.version.to_string()),
            ("ayah_count", ayah_ids.len().to_string()),
            ("rows", prepared.rows.len().to_string()),
            ("blocks", prepared.blocks.to_string()),
            ("covered_ayahs", prepared.covered.to_string()),
        ] {
            meta.execute(params![k, v])?;
        }
    }

    tx.execute(
        "INSERT INTO tafsir
            (slug, language, author, title, version, translator, name_native,
             direction, school, creed, source_url, license, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            edition.slug,
            edition.language,
            edition.author,
            edition.title,
            edition.version,
            edition.translator,
            edition.name_native,
            edition.direction,
            edition.school,
            edition.creed,
            edition.source_url,
            edition.license,
            edition.sort_order,
        ],
    )?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO tafsir_ayah (ayah_id, text, group_start_ayah_id, group_end_ayah_id)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (ayah_id, text, group_start, group_end) in &prepared.rows {
            stmt.execute(params![ayah_id, text, group_start, group_end])
                .with_context(|| format!("Inserting ayah id {ayah_id}"))?;
        }
    }
    tx.commit().context("Committing pack rows")?;

    // The pack is downloaded over a connection someone is paying for, so slack
    // pages are not free here either.
    pack.execute_batch("VACUUM;")?;
    drop(pack);

    let size = std::fs::metadata(out)?.len();
    log::info!(
        "  → {} — {} rows, {} blocks ({} runs, longest {}), {:.2} MB of text, {:.2} MB on disk",
        out.display(),
        prepared.rows.len(),
        prepared.blocks,
        prepared.runs,
        prepared.longest,
        text_bytes(&prepared.rows) as f64 / 1_048_576.0,
        size as f64 / 1_048_576.0,
    );
    log::info!("  → sha256 {}", sha256_file(out)?);

    Ok(())
}

/// Bytes of actual commentary, not characters: for an Arabic edition the two
/// differ by roughly half, and this is the number size decisions get made on.
fn text_bytes(rows: &[Row]) -> usize {
    rows.iter().map(|(_, t, _, _)| t.len()).sum()
}

/// Lowercase hex SHA-256 of a file, streamed rather than read whole — a pack
/// is tens of megabytes.
///
/// Printed at the end of `emit_pack` because it is what has to be copied into
/// the app's `PACKS` table: that constant is the only thing standing between a
/// download and the user's database, so it is produced by the same run that
/// produces the file it describes.
pub fn sha256_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file =
        std::fs::File::open(path).with_context(|| format!("Hashing {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
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

    fn entry(surah: u32, ayah: u32, text: &str) -> Entry {
        Entry {
            surah,
            ayah,
            text: text.to_string(),
        }
    }

    fn shape(groups: &[Group]) -> Vec<(u32, u32, u32, &str)> {
        groups
            .iter()
            .map(|g| (g.surah, g.start_ayah, g.end_ayah, g.text.as_str()))
            .collect()
    }

    #[test]
    fn collapses_a_repeated_block_into_one_run() {
        let groups = group_entries(vec![
            entry(2, 1, "on the opening verses"),
            entry(2, 2, "on the opening verses"),
            entry(2, 3, "on the opening verses"),
            entry(2, 4, "on verse four"),
        ]);
        assert_eq!(
            shape(&groups),
            vec![
                (2, 1, 3, "on the opening verses"),
                (2, 4, 4, "on verse four"),
            ]
        );
        assert!(groups[0].is_run());
        assert!(!groups[1].is_run());
    }

    #[test]
    fn a_gap_in_ayah_numbers_ends_the_run() {
        // The same block either side of a verse the edition passes over is two
        // blocks, not one spanning the gap — the run is stored as an id range,
        // and a range across the gap would claim the skipped verse too.
        let groups = group_entries(vec![
            entry(2, 1, "same text"),
            entry(2, 2, "same text"),
            entry(2, 5, "same text"),
        ]);
        assert_eq!(
            shape(&groups),
            vec![(2, 1, 2, "same text"), (2, 5, 5, "same text")]
        );
    }

    #[test]
    fn a_surah_boundary_ends_the_run() {
        let groups = group_entries(vec![entry(1, 7, "same text"), entry(2, 1, "same text")]);
        assert_eq!(
            shape(&groups),
            vec![(1, 7, 7, "same text"), (2, 1, 1, "same text")]
        );
    }

    #[test]
    fn text_differing_by_a_space_is_two_blocks() {
        let groups = group_entries(vec![entry(2, 1, "a b"), entry(2, 2, "a  b")]);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn a_per_ayah_edition_yields_one_group_per_entry() {
        let groups = group_entries(vec![
            entry(2, 1, "alif"),
            entry(2, 2, "lam"),
            entry(2, 3, "mim"),
        ]);
        assert_eq!(groups.len(), 3);
        assert!(groups.iter().all(|g| !g.is_run()));
    }
}
