use rusqlite::Connection;
use std::path::Path;
use crate::db::error::DbResult;

/// The full SQLite schema applied when no seed database is available.
/// Embedded at compile time from `database/schema.sql`.
const SCHEMA_SQL: &str = include_str!("../../../database/schema.sql");

/// The pre-populated Quran database (114 Surahs, 6,236 Ayahs) shipped with
/// the app. Embedded at compile time so first run works fully offline —
/// no separate import step required.
const SEED_DB: &[u8] = include_bytes!("../../../database/quran.db");

/// Current schema version expected by this build.
const CURRENT_VERSION: u32 = 2;

/// Open (or create) the SQLite database at the given path and ensure it is
/// at the expected schema version. Returns a configured [`Connection`].
///
/// # WAL mode & pragmas
/// The schema.sql already contains the PRAGMA statements, but we also set
/// them in code so they apply to every connection opened by the application.
pub fn open(path: &Path) -> DbResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        log::info!("No database found at {:?}; seeding from bundled quran.db", path);
        std::fs::write(path, SEED_DB)?;
    }

    let conn = Connection::open(path)?;
    configure_connection(&conn)?;

    let version = get_schema_version(&conn)?;
    if version == 0 {
        // Empty/corrupt file with no seed data — fall back to schema only.
        log::info!("Applying initial database schema (v{})", CURRENT_VERSION);
        conn.execute_batch(SCHEMA_SQL)?;
    } else if version < CURRENT_VERSION {
        // Future: run incremental migrations here
        log::info!(
            "Database at v{}, upgrading to v{}",
            version,
            CURRENT_VERSION
        );
        run_migrations(&conn, version)?;
    } else {
        log::info!("Database is at current schema v{}", version);
    }

    Ok(conn)
}

/// Apply per-connection SQLite pragmas for performance and correctness.
fn configure_connection(conn: &Connection) -> DbResult<()> {
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -8000;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;
    ")?;
    Ok(())
}

/// Read the recorded schema version (0 = brand new database).
fn get_schema_version(conn: &Connection) -> DbResult<u32> {
    // schema_version table may not exist yet on a brand new db
    let exists: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM sqlite_master
            WHERE type='table' AND name='schema_version'
        )",
        [],
        |row| row.get(0),
    )?;

    if !exists {
        return Ok(0);
    }

    let version: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    Ok(version)
}

/// Run incremental migrations from `from_version` up to `CURRENT_VERSION`.
/// Each migration is a separate SQL block; add new arms as the schema evolves.
fn run_migrations(conn: &Connection, from_version: u32) -> DbResult<()> {
    if from_version < 2 {
        log::info!("  → Applying migration 002: mushaf page layout");
        conn.execute_batch(include_str!("../../../database/migrations/002_mushaf_layout.sql"))?;
    }

    let version = get_schema_version(conn)?;
    if version < CURRENT_VERSION {
        log::warn!(
            "No migration path found from v{} to v{}. DB may be ahead of code.",
            version,
            CURRENT_VERSION
        );
    }
    Ok(())
}

/// Return basic stats useful for debugging / About screen.
pub fn stats(conn: &Connection) -> DbResult<DbStats> {
    let surah_count: u32 = conn.query_row("SELECT COUNT(*) FROM surah", [], |r| r.get(0))?;
    let ayah_count: u32 = conn.query_row("SELECT COUNT(*) FROM ayah", [], |r| r.get(0))?;
    let bookmark_count: u32 = conn.query_row("SELECT COUNT(*) FROM bookmark", [], |r| r.get(0))?;
    let note_count: u32 = conn.query_row("SELECT COUNT(*) FROM note", [], |r| r.get(0))?;

    Ok(DbStats {
        surah_count,
        ayah_count,
        bookmark_count,
        note_count,
        schema_version: get_schema_version(conn)?,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct DbStats {
    pub surah_count: u32,
    pub ayah_count: u32,
    pub bookmark_count: u32,
    pub note_count: u32,
    pub schema_version: u32,
}
