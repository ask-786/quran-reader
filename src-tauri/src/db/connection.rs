use crate::db::error::DbResult;
use rusqlite::{params, Connection};
use std::path::Path;

/// The full SQLite schema applied when no seed database is available.
/// Embedded at compile time from `database/schema.sql`.
const SCHEMA_SQL: &str = include_str!("../../../database/schema.sql");

/// The pre-populated Quran database (114 Surahs, 6,236 Ayahs) shipped with
/// the app. Embedded at compile time so first run works fully offline —
/// no separate import step required.
const SEED_DB: &[u8] = include_bytes!("../../../database/quran.db");

/// Current schema version expected by this build.
const CURRENT_VERSION: u32 = 6;

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
        log::info!(
            "No database found at {:?}; seeding from bundled quran.db",
            path
        );
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
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -8000;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;
    ",
    )?;
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
///
/// **Every schema change runs before any data rebuild**, and the two phases
/// are kept apart deliberately. `rebuild_mushaf_layout_from_seed` copies with
/// `INSERT ... SELECT *`, which matches columns by position, so it is only
/// correct once the local `page_line_word` has the same shape as the seed's.
/// Interleaving the two broke exactly that: with the rebuild sitting inside
/// the 003 arm, a v2 install would rebuild while it still had the `glyph_v2`
/// that 004 drops and the seed no longer carries — an 8-column table fed from
/// a 7-column select.
fn run_migrations(conn: &Connection, from_version: u32) -> DbResult<()> {
    if from_version < 2 {
        log::info!("  → Applying migration 002: mushaf page layout");
        conn.execute_batch(include_str!(
            "../../../database/migrations/002_mushaf_layout.sql"
        ))?;
    }

    if from_version < 3 {
        log::info!("  → Applying migration 003: QCF v4 glyphs");
        conn.execute_batch(include_str!("../../../database/migrations/003_qcf_v4.sql"))?;
    }

    if from_version < 4 {
        log::info!("  → Applying migration 004: drop QCF v2 glyphs");
        conn.execute_batch(include_str!(
            "../../../database/migrations/004_drop_glyph_v2.sql"
        ))?;
    }

    if from_version < 5 {
        log::info!("  → Applying migration 005: rub-el-hizb ornament glyphs");
        conn.execute_batch(include_str!(
            "../../../database/migrations/005_rub_el_hizb_glyphs.sql"
        ))?;
    }

    if from_version < 6 {
        log::info!("  → Applying migration 006: reading position & history");
        conn.execute_batch(include_str!(
            "../../../database/migrations/006_reading_history.sql"
        ))?;
    }

    // page_line/page_line_word carry no user data (bookmarks and notes key off
    // ayah_id, not page/line/glyph), so glyph content is delivered by
    // rebuilding both tables wholesale from the bundled seed rather than by
    // re-deriving anything in place. 003 needed it because its ALTER left
    // every glyph_v4 NULL; 005 needs it because the seed now carries 199
    // rub-el-hizb ornaments the old one didn't.
    //
    // One rebuild covers both — an install coming from v2 would otherwise do
    // the same wholesale copy twice on the same upgrade.
    if from_version < 5 {
        log::info!("  → Rebuilding page_line/page_line_word from the bundled seed");
        rebuild_mushaf_layout_from_seed(conn)?;
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

/// Replace `page_line`/`page_line_word` wholesale with the copies from the
/// embedded `SEED_DB`, by writing it to a scratch file and `ATTACH`ing it —
/// SQLite has no cross-database `INSERT ... SELECT` without a filesystem
/// path. Assumes the seed's `ayah` table (and therefore its `ayah.id` values)
/// matches the target's, which already holds for every existing install:
/// Tanzil/Surah-metadata import is deterministic, and no migration before
/// this one has ever touched `ayah`.
fn rebuild_mushaf_layout_from_seed(conn: &Connection) -> DbResult<()> {
    // Unique per call, not just per process: the pid alone collides when two
    // upgrades run concurrently in one process, which is exactly what the
    // tests below do.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut seed_path = std::env::temp_dir();
    seed_path.push(format!(
        "quranreader-seed-{}-{nonce}.db",
        std::process::id()
    ));
    std::fs::write(&seed_path, SEED_DB)?;

    let result = (|| -> DbResult<()> {
        let seed_path_str = seed_path.to_string_lossy().to_string();
        conn.execute("ATTACH DATABASE ?1 AS seed", params![seed_path_str])?;

        let copy_result = (|| -> DbResult<()> {
            conn.execute_batch(
                "BEGIN;
                 DELETE FROM page_line_word;
                 DELETE FROM page_line;
                 INSERT INTO page_line SELECT * FROM seed.page_line;
                 INSERT INTO page_line_word SELECT * FROM seed.page_line_word;
                 COMMIT;",
            )?;
            Ok(())
        })();

        conn.execute_batch("DETACH DATABASE seed;")?;
        copy_result
    })();

    let _ = std::fs::remove_file(&seed_path);
    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    /// Reshape a copy of `SEED_DB` into what a v0.1.1 install's database looks
    /// like on disk: schema v2, `page_line_word` carrying `glyph_v2` and no
    /// `glyph_v4`. Faithful because v0.1.1's `ayah`, `surah` and `page_line`
    /// tables are byte-identical to the current seed's (verified by EXCEPT in
    /// both directions), so only `page_line_word`'s shape actually differs.
    ///
    /// Set `QURAN_TEST_V2_DB` to run against a genuine v0.1.1 file instead —
    /// `git show v0.1.1:database/quran.db > /tmp/v011.db`.
    fn v011_install(path: &Path) {
        if let Ok(real) = std::env::var("QURAN_TEST_V2_DB") {
            std::fs::copy(real, path).unwrap();
        } else {
            std::fs::write(path, SEED_DB).unwrap();
            let conn = Connection::open(path).unwrap();
            conn.execute_batch(
                "ALTER TABLE page_line_word DROP COLUMN glyph_v4;
                 ALTER TABLE page_line_word ADD COLUMN glyph_v2 TEXT NOT NULL DEFAULT 'x';
                 DELETE FROM schema_version WHERE version >= 3;",
            )
            .unwrap();
        }
    }

    /// The upgrade path an AUR user takes from the v0.1.1 release. This is the
    /// case that can silently produce a blank Mushaf: `connection.rs` only
    /// seeds when the file is absent, so an upgrading user keeps their own
    /// database and every glyph they render has to come from a migration.
    #[test]
    fn upgrade_from_v011_populates_v4_glyphs_and_keeps_user_data() {
        let dir = std::env::temp_dir().join(format!("quranreader-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);
        v011_install(&path);

        // Ayat al-Kursi — a stable, easily recognised verse to prove the
        // bookmark still points at the same words after the layout tables have
        // been deleted and rebuilt underneath it.
        let (kursi_id, before_version): (i64, u32) = {
            let conn = Connection::open(&path).unwrap();
            let id: i64 = conn
                .query_row(
                    "SELECT id FROM ayah WHERE surah_id = 2 AND ayah_number = 255",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            conn.execute(
                "INSERT INTO bookmark (ayah_id, label) VALUES (?1, 'kursi')",
                params![id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO note (ayah_id, content) VALUES (?1, 'my note')",
                params![id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('last_read_ayah_id', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![id.to_string()],
            )
            .unwrap();
            (id, get_schema_version(&conn).unwrap())
        };
        assert_eq!(
            before_version, 2,
            "fixture should start as a v0.1.1 install"
        );

        // The upgrade itself, through the real entry point.
        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);

        let cols: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('page_line_word')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(!cols.iter().any(|c| c == "glyph_v2"), "glyph_v2 dropped");
        assert!(cols.iter().any(|c| c == "glyph_v4"), "glyph_v4 present");

        // A blank Mushaf is precisely "rows exist but glyph_v4 is NULL", so
        // assert on the glyph data rather than just the row count.
        let (rows, null_v4): (u32, u32) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE glyph_v4 IS NULL) FROM page_line_word",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 77_539);
        assert_eq!(null_v4, 0, "every word row must render");

        // Migration 005: the rub-el-hizb ornament reaches an existing install.
        // `uthmani_text` has always carried ۞ on these 199 rows; before 005 the
        // glyph beside it drew only the word, so the ornament never rendered.
        // A glyph pair ("<ornament> <word>") is what fixes that, and only the
        // seed rebuild can deliver it.
        let ornaments: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_line_word
                 WHERE uthmani_text LIKE '%' || char(1758) || '%'
                   AND glyph_v4 LIKE '% %'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ornaments, 199, "every ۞ must have a glyph to render with");

        // User data survives, and still resolves to the same verse.
        let (b_surah, b_ayah): (u32, u32) = conn
            .query_row(
                "SELECT a.surah_id, a.ayah_number FROM bookmark b
                 JOIN ayah a ON a.id = b.ayah_id WHERE b.label = 'kursi'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            (b_surah, b_ayah),
            (2, 255),
            "bookmark still on Ayat al-Kursi"
        );

        let note: String = conn
            .query_row(
                "SELECT content FROM note WHERE ayah_id = ?1",
                params![kursi_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(note, "my note");

        // Migration 006 moves the reading position out of `settings` and into
        // `reading_position`, keyed by the range it was read in. The Surah is
        // re-derived from the Ayah rather than taken from the old companion
        // key, so it is right even if the two had drifted apart.
        let (scope, scope_id, position): (String, u32, i64) = conn
            .query_row(
                "SELECT scope, scope_id, ayah_id FROM reading_position",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (scope.as_str(), scope_id, position),
            ("surah", 2, kursi_id),
            "reading position survives, as a Surah-scoped position"
        );
        let orphaned: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings
                 WHERE key IN ('last_read_surah_id', 'last_read_ayah_id', 'scroll_position')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orphaned, 0, "the retired settings keys are gone");

        // And the bookmarked verse actually has glyphs to render.
        let kursi_words: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_line_word WHERE ayah_id = ?1 AND glyph_v4 IS NOT NULL",
                params![kursi_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(kursi_words > 0, "bookmarked ayah renders");

        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        let mut fk = conn.prepare("PRAGMA foreign_key_check").unwrap();
        assert!(
            fk.query([]).unwrap().next().unwrap().is_none(),
            "no dangling ayah_id references after the layout rebuild"
        );
        drop(fk);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The other half of the release: someone installing for the first time,
    /// who gets `SEED_DB` written out whole and must skip migrations entirely.
    #[test]
    fn fresh_install_seeds_at_current_version() {
        let dir = std::env::temp_dir().join(format!("quranreader-fresh-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("quran.db");
        assert!(!path.exists());

        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);
        let (rows, null_v4): (u32, u32) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE glyph_v4 IS NULL) FROM page_line_word",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 77_539);
        assert_eq!(null_v4, 0);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// v0.1.0 predates the Mushaf layout entirely — its database has no
    /// `page_line`/`page_line_word` at all, so 002 creates them empty and the
    /// rebuild is the only thing that ever fills them.
    #[test]
    fn upgrade_from_v010_creates_and_fills_layout_tables() {
        let dir = std::env::temp_dir().join(format!("quranreader-v010-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);

        std::fs::write(&path, SEED_DB).unwrap();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "DROP TABLE page_line_word;
                 DROP TABLE page_line;
                 DELETE FROM schema_version WHERE version >= 2;",
            )
            .unwrap();
            assert_eq!(get_schema_version(&conn).unwrap(), 1);
        }

        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);
        let (rows, null_v4): (u32, u32) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE glyph_v4 IS NULL) FROM page_line_word",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 77_539);
        assert_eq!(null_v4, 0);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
