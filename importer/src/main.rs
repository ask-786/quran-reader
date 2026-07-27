mod fetch;
mod insert;
mod mushaf;
mod parse;
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
