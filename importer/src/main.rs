mod fetch;
mod parse;
mod validate;
mod insert;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Output database path (relative to the workspace root, resolved at runtime).
fn db_output_path() -> PathBuf {
    // When run from `importer/` via `cargo run`, go up one level.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("database")
        .join("quran.db")
}

fn schema_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("database")
        .join("schema.sql")
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    log::info!("=== Quran Importer ===");
    log::info!("Phase 4 — Import Pipeline");
    log::info!("");

    // -----------------------------------------------------------------------
    // Step 1 — Fetch raw data from Tanzil
    // -----------------------------------------------------------------------
    log::info!("[1/4] Fetching Tanzil data …");
    let raw = fetch::fetch_all().context("Failed to fetch Tanzil data")?;

    // -----------------------------------------------------------------------
    // Step 2 — Parse
    // -----------------------------------------------------------------------
    log::info!("[2/4] Parsing XML and metadata …");
    let quran = parse::parse(&raw).context("Failed to parse Quran data")?;

    log::info!(
        "      Parsed {} surahs, {} ayahs",
        quran.surahs.len(),
        quran.ayahs.len()
    );

    // -----------------------------------------------------------------------
    // Step 3 — Validate
    // -----------------------------------------------------------------------
    log::info!("[3/4] Validating …");
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
    log::info!("[4/4] Writing database to {} …", db_path.display());

    // Remove stale database so we start clean
    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .context("Failed to remove old quran.db")?;
        log::info!("      Removed existing quran.db");
    }

    insert::write_db(&db_path, &schema_path, &quran)
        .context("Failed to write database")?;

    log::info!("");
    log::info!("=== Import complete ===");
    log::info!("Database: {}", db_path.display());

    Ok(())
}
