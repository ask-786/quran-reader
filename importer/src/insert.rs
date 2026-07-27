use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::parse::QuranData;

/// Write the full Quran dataset to a new SQLite database.
///
/// Steps:
///   1. Apply the schema (schema.sql)
///   2. Insert all Surahs
///   3. Insert all Ayahs
///   4. Rebuild FTS index
///   5. Run integrity check
pub fn write_db(db_path: &Path, schema_path: &Path, quran: &QuranData) -> Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let schema_sql = std::fs::read_to_string(schema_path)
        .with_context(|| format!("Reading schema from {}", schema_path.display()))?;

    let conn = Connection::open(db_path).context("Opening output database")?;

    // Apply per-connection pragmas
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -32000;
        PRAGMA temp_store = MEMORY;
    ",
    )?;

    // Apply schema
    log::info!("  → Applying schema …");
    conn.execute_batch(&schema_sql)
        .context("Applying database schema")?;

    // -----------------------------------------------------------------------
    // Insert Surahs
    // -----------------------------------------------------------------------
    log::info!("  → Inserting {} Surahs …", quran.surahs.len());
    {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO surah
             (id, name_ar, name_en, transliteration, revelation_type,
              verses_count, order_of_revelation, has_bismillah)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for s in &quran.surahs {
            stmt.execute(params![
                s.id,
                s.name_ar,
                s.name_en,
                s.transliteration,
                s.revelation_type,
                s.verses_count,
                s.order_of_revelation,
                s.has_bismillah as i64,
            ])
            .with_context(|| format!("Inserting surah {}", s.id))?;
        }
    }

    // -----------------------------------------------------------------------
    // Insert Ayahs (in one transaction for speed)
    // -----------------------------------------------------------------------
    log::info!(
        "  → Inserting {} Ayahs (this may take a moment) …",
        quran.ayahs.len()
    );
    {
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO ayah
                 (surah_id, ayah_number, uthmani_text, simple_text,
                  juz, hizb, rub_hizb, manzil, ruku, page, sajdah)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?;

            for (i, a) in quran.ayahs.iter().enumerate() {
                stmt.execute(params![
                    a.surah_id,
                    a.ayah_number,
                    a.uthmani_text,
                    a.simple_text,
                    a.juz,
                    a.hizb,
                    a.rub_hizb,
                    a.manzil,
                    a.ruku,
                    a.page,
                    a.sajdah as i64,
                ])
                .with_context(|| format!("Inserting ayah {}:{}", a.surah_id, a.ayah_number))?;

                if (i + 1) % 1000 == 0 {
                    log::info!("      … {}/{}", i + 1, quran.ayahs.len());
                }
            }
        }

        tx.commit().context("Committing ayah transaction")?;
    }

    // -----------------------------------------------------------------------
    // Rebuild FTS index
    // -----------------------------------------------------------------------
    log::info!("  → Rebuilding FTS index …");
    conn.execute_batch("INSERT INTO fts_ayah(fts_ayah) VALUES ('rebuild');")?;

    // -----------------------------------------------------------------------
    // Integrity check
    // -----------------------------------------------------------------------
    log::info!("  → Running integrity check …");
    let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result != "ok" {
        anyhow::bail!("Integrity check failed: {result}");
    }

    // -----------------------------------------------------------------------
    // Final stats
    // -----------------------------------------------------------------------
    let surah_count: u32 = conn.query_row("SELECT COUNT(*) FROM surah", [], |r| r.get(0))?;
    let ayah_count: u32 = conn.query_row("SELECT COUNT(*) FROM ayah", [], |r| r.get(0))?;
    let fts_count: u32 = conn.query_row("SELECT COUNT(*) FROM fts_ayah", [], |r| r.get(0))?;

    log::info!("  → Final DB stats:");
    log::info!("      Surahs:    {}", surah_count);
    log::info!("      Ayahs:     {}", ayah_count);
    log::info!("      FTS rows:  {}", fts_count);

    // Report file size
    if let Ok(meta) = std::fs::metadata(db_path) {
        let size_kb = meta.len() / 1024;
        log::info!(
            "      DB size:   {} KB ({:.1} MB)",
            size_kb,
            size_kb as f64 / 1024.0
        );
    }

    Ok(())
}
