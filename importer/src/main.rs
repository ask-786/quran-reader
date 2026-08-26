mod fetch;
mod insert;
mod mushaf;
mod parse;
mod tafsir;
mod validate;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Output database path (relative to the workspace root, resolved at runtime).
fn db_output_path() -> PathBuf {
    // When run from `importer/` via `cargo run`, go up one level.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("database")
        .join("quran.db")
}

fn schema_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("database")
        .join("schema.sql")
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if std::env::args().any(|a| a == "--repair-mushaf-headers") {
        let db_path = db_output_path();
        log::info!("=== Repairing Mushaf surah_header anchoring ===");
        log::info!("Database: {}", db_path.display());
        mushaf::repair_surah_headers(&db_path).context("Failed to repair surah headers")?;
        log::info!("=== Repair complete ===");
        return Ok(());
    }

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--list-tafsir") {
        log::info!("Available tafsir editions:");
        for e in tafsir::EDITIONS {
            log::info!(
                "  {:<24} {} ({}) — {} · {}",
                e.slug,
                e.title,
                e.language,
                e.school,
                e.creed
            );
        }
        return Ok(());
    }

    // `--emit-pack <slug> [--pack-out <dir>] [--tafsir-dir <path>]` — build a
    // downloadable content pack instead of writing into the seed database.
    // This is how every edition that does not ship in the installer reaches an
    // install; see `emit_pack`. It prints the pack's SHA-256, which is what
    // goes into the app's PACKS table.
    if let Some(slug) = args
        .iter()
        .position(|a| a == "--emit-pack")
        .and_then(|i| args.get(i + 1))
    {
        let db_path = db_output_path();
        let edition = tafsir::find_edition(slug).with_context(|| {
            format!("Unknown tafsir edition '{slug}' — run with --list-tafsir to see the editions this reader carries")
        })?;

        let out_dir = args
            .iter()
            .position(|a| a == "--pack-out")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("packs"));
        std::fs::create_dir_all(&out_dir)
            .with_context(|| format!("Creating {}", out_dir.display()))?;
        let out = out_dir.join(format!("{}-v{}.qpack", edition.slug, edition.version));

        log::info!("=== Emitting pack: {} ===", edition.title);
        log::info!("Source database: {}", db_path.display());

        let entries = match tafsir::local_dir_arg(&args) {
            Some(dir) => {
                log::info!("[1/2] Loading {} from {} …", edition.slug, dir.display());
                tafsir::load_edition(&dir)?
            }
            None => {
                log::info!("[1/2] Fetching {} (114 surahs) …", edition.slug);
                tafsir::fetch_edition(edition)?
            }
        };

        log::info!("[2/2] Writing pack …");
        tafsir::emit_pack(&db_path, edition, entries, &out).context("Failed to emit pack")?;

        log::info!("=== Pack complete ===");
        return Ok(());
    }

    // `--import-tafsir <slug> [--bundle] [--tafsir-dir <path>]`. `--bundle`
    // marks the edition as shipping with the app, which is only true for the
    // seed database this repo commits — see write_tafsir.
    if let Some(slug) = args
        .iter()
        .position(|a| a == "--import-tafsir")
        .and_then(|i| args.get(i + 1))
    {
        let db_path = db_output_path();
        let edition = tafsir::find_edition(slug).with_context(|| {
            format!("Unknown tafsir edition '{slug}' — run with --list-tafsir to see the editions this reader carries")
        })?;
        let bundled = args.iter().any(|a| a == "--bundle");

        log::info!("=== Importing tafsir: {} ===", edition.title);
        log::info!("Database: {}", db_path.display());

        let entries = match tafsir::local_dir_arg(&args) {
            Some(dir) => {
                log::info!("[1/2] Loading {} from {} …", edition.slug, dir.display());
                tafsir::load_edition(&dir)?
            }
            None => {
                log::info!("[1/2] Fetching {} (114 surahs) …", edition.slug);
                tafsir::fetch_edition(edition)?
            }
        };

        log::info!("[2/2] Writing {} entries …", entries.len());
        tafsir::write_tafsir(&db_path, edition, entries, bundled)
            .context("Failed to write tafsir")?;

        log::info!("=== Tafsir import complete ===");
        return Ok(());
    }

    if std::env::args().any(|a| a == "--import-mushaf-v4") {
        let db_path = db_output_path();
        log::info!("=== Importing QCF v4 glyphs ===");
        log::info!("Database: {}", db_path.display());

        // `--mushaf-v4-dir <path>` reads the layout from a local directory of
        // 001.json … 604.json instead of making 604 sequential requests. Get
        // one with:  npm pack quran-qcf4 && tar xzf quran-qcf4-*.tgz
        // then point at `package/pages`.
        let local_dir = std::env::args()
            .skip_while(|a| a != "--mushaf-v4-dir")
            .nth(1)
            .map(PathBuf::from);

        let pages = match local_dir {
            Some(dir) => {
                log::info!("[1/2] Loading QCF v4 page layout from {} …", dir.display());
                mushaf::load_all_pages_v4(&dir).context("Failed to load QCF v4 layout")?
            }
            None => {
                log::info!("[1/2] Fetching QCF v4 page layout (604 pages) …");
                mushaf::fetch_all_pages_v4().context("Failed to fetch QCF v4 layout")?
            }
        };
        log::info!("[2/2] Writing glyph_v4 onto existing page_line_word rows …");
        mushaf::write_glyph_v4(&db_path, &pages).context("Failed to write QCF v4 glyphs")?;
        log::info!("=== QCF v4 import complete ===");
        return Ok(());
    }

    log::info!("=== Quran Importer ===");
    log::info!("Phase 4 — Import Pipeline");
    log::info!("");

    // -----------------------------------------------------------------------
    // Step 1 — Fetch raw data from Tanzil
    // -----------------------------------------------------------------------
    log::info!("[1/6] Fetching Tanzil data …");
    let raw = fetch::fetch_all().context("Failed to fetch Tanzil data")?;

    // -----------------------------------------------------------------------
    // Step 2 — Parse
    // -----------------------------------------------------------------------
    log::info!("[2/6] Parsing XML and metadata …");
    let quran = parse::parse(&raw).context("Failed to parse Quran data")?;

    log::info!(
        "      Parsed {} surahs, {} ayahs",
        quran.surahs.len(),
        quran.ayahs.len()
    );

    // -----------------------------------------------------------------------
    // Step 3 — Validate
    // -----------------------------------------------------------------------
    log::info!("[3/6] Validating …");
    validate::validate(&quran).context("Validation failed")?;
    log::info!("      ✓ 114 Surahs");
    log::info!("      ✓ 6236 Ayahs");
    log::info!("      ✓ All Juz (1–30)");
    log::info!("      ✓ All Hizb (1–60)");
    log::info!("      ✓ All Pages (1–604)");
    log::info!("      ✓ Sajdah count");

    // -----------------------------------------------------------------------
    // Step 4 — Write SQLite
    // -----------------------------------------------------------------------
    let db_path = db_output_path();
    let schema_path = schema_path();
    log::info!("[4/6] Writing database to {} …", db_path.display());

    // Remove stale database so we start clean
    if db_path.exists() {
        std::fs::remove_file(&db_path).context("Failed to remove old quran.db")?;
        log::info!("      Removed existing quran.db");
    }

    insert::write_db(&db_path, &schema_path, &quran).context("Failed to write database")?;

    // -----------------------------------------------------------------------
    // Step 5 — Fetch Mushaf page layout (line-by-line, QCF v2 glyphs)
    // -----------------------------------------------------------------------
    log::info!("[5/6] Fetching Mushaf page layout (604 pages) …");
    let pages = mushaf::fetch_all_pages().context("Failed to fetch mushaf layout")?;

    // -----------------------------------------------------------------------
    // Step 6 — Insert Mushaf page layout
    // -----------------------------------------------------------------------
    log::info!("[6/6] Writing Mushaf page layout …");
    mushaf::write_mushaf_layout(&db_path, &pages).context("Failed to write mushaf layout")?;

    log::info!("");
    log::info!("=== Import complete ===");
    log::info!("Database: {}", db_path.display());

    Ok(())
}
