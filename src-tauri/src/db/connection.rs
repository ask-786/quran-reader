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
const CURRENT_VERSION: u32 = 8;

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

    if from_version < 7 {
        log::info!("  → Applying migration 007: tafsir metadata, grouping, FTS");
        conn.execute_batch(include_str!("../../../database/migrations/007_tafsir.sql"))?;
    }

    if from_version < 8 {
        log::info!("  → Applying migration 008: QCF v4 Mushaf page layout");
        conn.execute_batch(include_str!(
            "../../../database/migrations/008_v4_layout_rebuild.sql"
        ))?;
    }

    // page_line/page_line_word carry no user data (bookmarks and notes key off
    // ayah_id, not page/line/glyph), so layout content is delivered by
    // rebuilding both tables wholesale from the bundled seed rather than by
    // re-deriving anything in place. 003 needed it because its ALTER left
    // every glyph_v4 NULL; 005 needed it for the 199 rub-el-hizb ornaments;
    // 008 needs it because the whole layout is now built from the v4 source
    // rather than joined across two that disagree.
    //
    // One rebuild covers all of them — an install coming from v2 would
    // otherwise do the same wholesale copy three times on the same upgrade.
    // The gate has to track the *newest* of those migrations: at `< 5` an
    // install already at v7 would never re-run it, and 008 delivers nothing
    // but this copy.
    if from_version < 8 {
        log::info!("  → Rebuilding page_line/page_line_word from the bundled seed");
        rebuild_mushaf_layout_from_seed(conn)?;
    }

    // No equivalent step for tafsir, deliberately. The glyphs above are copied
    // out of the seed because they ship with the app; no tafsir does. Every
    // edition arrives as a downloaded pack instead (see `packs`), so an
    // upgrading install has nothing to be given here and 007's reshaping of
    // the empty tables is the whole of the work.

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
    with_seed_attached(conn, |conn| {
        conn.execute_batch(
            "BEGIN;
             DELETE FROM page_line_word;
             DELETE FROM page_line;
             INSERT INTO page_line SELECT * FROM seed.page_line;
             INSERT INTO page_line_word SELECT * FROM seed.page_line_word;
             COMMIT;",
        )?;
        Ok(())
    })
}

/// Write `SEED_DB` to a scratch file, `ATTACH` it as `seed`, run `f`, then
/// detach and clean up whether or not `f` succeeded. SQLite has no
/// cross-database `INSERT ... SELECT` without a filesystem path, which is the
/// only reason the scratch file exists.
///
/// Every caller assumes the seed's `ayah` table (and therefore its `ayah.id`
/// values) matches the target's. That holds for every existing install: the
/// Tanzil/Surah-metadata import is deterministic and no migration has ever
/// touched `ayah`.
fn with_seed_attached<F>(conn: &Connection, f: F) -> DbResult<()>
where
    F: FnOnce(&Connection) -> DbResult<()>,
{
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

        let inner = f(conn);

        conn.execute_batch("DETACH DATABASE seed;")?;
        inner
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
    use rusqlite::{params, OptionalExtension};

    /// Reshape a copy of `SEED_DB` into what a v0.1.1 install's database looks
    /// like on disk: schema v2, `page_line_word` carrying `glyph_v2` and no
    /// `glyph_v4`, no reading-position tables, and no trace of migration 007 —
    /// no edition-metadata columns, no tafsir content, no `fts_tafsir`.
    /// Faithful because v0.1.1's `ayah`, `surah` and `page_line` tables are
    /// byte-identical to the current seed's (verified by EXCEPT in both
    /// directions), so only the shapes undone here actually differ.
    ///
    /// Undoing 006 and 007 is what makes the fixture exercise them at all: the
    /// seed ships fully migrated now, and `ALTER TABLE ADD COLUMN` on a column
    /// that already exists is an error, so a fixture that kept them would fail
    /// the upgrade for a reason no real install can hit.
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
            undo_007(&conn);
            undo_006(&conn);
        }
    }

    /// Undo migration 006 on a copy of the seed, so a fixture standing in for
    /// a pre-006 install actually looks like one — no reading-position tables,
    /// and the three retired `settings` keys back at their v1 defaults, which
    /// is what 006 reads the old position out of.
    fn undo_006(conn: &Connection) {
        conn.execute_batch(
            "DROP TABLE IF EXISTS reading_session;
             DROP TABLE IF EXISTS reading_position;
             INSERT OR IGNORE INTO settings (key, value) VALUES
                 ('last_read_surah_id', '1'),
                 ('last_read_ayah_id', '1'),
                 ('scroll_position', '0');
             DELETE FROM schema_version WHERE version >= 6;",
        )
        .unwrap();
    }

    /// Undo migration 007 on a copy of the seed, so a fixture standing in for
    /// a pre-007 install actually looks like one.
    ///
    /// Every fixture built from `SEED_DB` needs this, not just the tafsir test:
    /// the seed carries 007's columns now, and `ALTER TABLE ADD COLUMN` on an
    /// existing column is an error, so without this the migration fails for a
    /// reason no real install can hit.
    fn undo_007(conn: &Connection) {
        conn.execute_batch(
            "DELETE FROM tafsir_ayah;
             DELETE FROM tafsir;
             DROP TRIGGER IF EXISTS fts_tafsir_insert;
             DROP TRIGGER IF EXISTS fts_tafsir_delete;
             DROP TRIGGER IF EXISTS fts_tafsir_update;
             DROP TABLE IF EXISTS fts_tafsir;
             DROP TRIGGER IF EXISTS fts_translation_insert;
             DROP TRIGGER IF EXISTS fts_translation_delete;
             DROP TRIGGER IF EXISTS fts_translation_update;
             DROP INDEX IF EXISTS idx_tafsir_slug;
             DROP INDEX IF EXISTS idx_translation_slug;
             ALTER TABLE tafsir_ayah DROP COLUMN group_start_ayah_id;
             ALTER TABLE tafsir_ayah DROP COLUMN group_end_ayah_id;
             ALTER TABLE tafsir DROP COLUMN slug;
             ALTER TABLE tafsir DROP COLUMN translator;
             ALTER TABLE tafsir DROP COLUMN name_native;
             ALTER TABLE tafsir DROP COLUMN direction;
             ALTER TABLE tafsir DROP COLUMN school;
             ALTER TABLE tafsir DROP COLUMN creed;
             ALTER TABLE tafsir DROP COLUMN source_url;
             ALTER TABLE tafsir DROP COLUMN license;
             ALTER TABLE tafsir DROP COLUMN sort_order;
             ALTER TABLE translation DROP COLUMN slug;
             ALTER TABLE translation DROP COLUMN name_native;
             ALTER TABLE translation DROP COLUMN direction;
             ALTER TABLE translation DROP COLUMN school;
             ALTER TABLE translation DROP COLUMN creed;
             ALTER TABLE translation DROP COLUMN source_url;
             ALTER TABLE translation DROP COLUMN license;
             ALTER TABLE translation DROP COLUMN sort_order;
             DELETE FROM schema_version WHERE version >= 7;",
        )
        .unwrap();
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
        assert_eq!(rows, 77_545);
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

    /// The upgrade every v0.1.7 user takes: that release shipped schema v5, so
    /// an installed copy is sitting at v5 with only 006 and 007 between it and
    /// this build. It is the shortest path in the table and the most
    /// travelled, which is exactly why it is worth pinning: the reading
    /// position it carries across is the one piece of user data 006 touches.
    #[test]
    fn upgrade_from_v017_carries_the_reading_position_across() {
        let dir = std::env::temp_dir().join(format!("quranreader-v017-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);

        // A v0.1.7 install: the bundled seed wound back to v4 and 005 applied,
        // which is what that release's own `open()` would have left on disk.
        // The seed ships fully migrated now, so the winding back is what makes
        // this a v5 fixture rather than a copy of the current schema.
        std::fs::write(&path, SEED_DB).unwrap();
        let kursi_id: i64 = {
            let conn = Connection::open(&path).unwrap();
            undo_007(&conn);
            undo_006(&conn);
            conn.execute_batch(include_str!(
                "../../../database/migrations/005_rub_el_hizb_glyphs.sql"
            ))
            .unwrap();
            assert_eq!(
                get_schema_version(&conn).unwrap(),
                5,
                "fixture starts at v5"
            );

            let id: i64 = conn
                .query_row(
                    "SELECT id FROM ayah WHERE surah_id = 2 AND ayah_number = 255",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            conn.execute(
                "UPDATE settings SET value = ?1 WHERE key = 'last_read_ayah_id'",
                params![id.to_string()],
            )
            .unwrap();
            // Deliberately disagreeing with the Ayah above: nothing ever kept
            // the two keys in step, so 006 re-derives the Surah rather than
            // trusting this one.
            conn.execute(
                "UPDATE settings SET value = '9' WHERE key = 'last_read_surah_id'",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO bookmark (ayah_id, label) VALUES (?1, 'kursi')",
                params![id],
            )
            .unwrap();
            id
        };

        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);

        let (scope, scope_id, ayah_id): (String, u32, i64) = conn
            .query_row(
                "SELECT scope, scope_id, ayah_id FROM reading_position",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (scope.as_str(), scope_id, ayah_id),
            ("surah", 2, kursi_id),
            "the position becomes a Surah-scoped one, on the Surah the Ayah is in"
        );

        // Nothing has been read since the upgrade, so there are no sittings yet
        // — a position is where you are, a session is what you did.
        let sessions: u32 = conn
            .query_row("SELECT COUNT(*) FROM reading_session", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sessions, 0);

        let retired: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings
                 WHERE key IN ('last_read_surah_id', 'last_read_ayah_id', 'scroll_position')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(retired, 0, "the retired keys are gone");

        // The rest of the install is untouched: 006 and 007 are additive, and the layout
        // rebuild must not fire again for a database that already has v5 data.
        let bookmarks: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM bookmark WHERE label = 'kursi'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(bookmarks, 1);
        let (rows, null_v4): (u32, u32) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE glyph_v4 IS NULL) FROM page_line_word",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 77_545);
        assert_eq!(null_v4, 0);

        // Opening again is a no-op rather than a second application of 006.
        drop(conn);
        let conn = open(&path).unwrap();
        let positions: u32 = conn
            .query_row("SELECT COUNT(*) FROM reading_position", [], |r| r.get(0))
            .unwrap();
        assert_eq!(positions, 1, "relaunching does not re-run the migration");

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
        assert_eq!(rows, 77_545);
        assert_eq!(null_v4, 0);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The upgrade this release exists for. A v0.2.2 install is
    /// schema-identical to this build — 008 changes no table — so bumping the
    /// version is the *only* thing that can hand it the rebuilt layout. Put
    /// the seed-rebuild gate back to `< 5`, where it sat until 008, and this
    /// is the test that fails while every other upgrade test still passes.
    #[test]
    fn upgrade_from_v7_rebuilds_the_mushaf_layout() {
        let dir = std::env::temp_dir().join(format!("quranreader-v7-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);

        // The current seed wound back to v7 and damaged the way the old
        // two-source importer damaged it: 37:130 without its last word or its
        // ﴿١٣٠﴾ marker, and Surahs 81 and 85 with no Basmala line at all.
        std::fs::write(&path, SEED_DB).unwrap();
        let yaseen_id: i64 = {
            let conn = Connection::open(&path).unwrap();
            let id: i64 = conn
                .query_row(
                    "SELECT id FROM ayah WHERE surah_id = 37 AND ayah_number = 130",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            conn.execute(
                "DELETE FROM page_line_word WHERE ayah_id = ?1 AND word_index = 4",
                params![id],
            )
            .unwrap();
            conn.execute(
                "UPDATE page_line_word SET glyph_v4 = 'x'
                 WHERE ayah_id = ?1 AND word_index = 3",
                params![id],
            )
            .unwrap();
            conn.execute_batch(
                "DELETE FROM page_line WHERE line_type = 'basmala' AND page IN (586, 590);
                 DELETE FROM schema_version WHERE version >= 8;",
            )
            .unwrap();
            assert_eq!(
                get_schema_version(&conn).unwrap(),
                7,
                "fixture starts at v7"
            );
            id
        };

        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);

        // 37:130 is whole again, and its last word carries the marker as a
        // second glyph.
        let (words, marked): (u32, u32) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(*) FILTER (WHERE glyph_v4 LIKE '% %')
                 FROM page_line_word WHERE ayah_id = ?1",
                params![yaseen_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(words, 4, "سَلَامٌ عَلَىٰ إِلْ يَاسِينَ");
        assert_eq!(marked, 1, "يَاسِينَ carries the ﴿١٣٠﴾ marker");

        // At-Takwir and Al-Burooj open with a Bismillah like every other Surah
        // that should have one.
        let basmalas: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_line WHERE line_type = 'basmala'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(basmalas, 112);

        // And the class of bug, not just the verse that exposed it: no Ayah
        // ends on a word without a marker glyph beside it.
        let unmarked: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_line_word w
                 JOIN (SELECT ayah_id, MAX(word_index) AS mw FROM page_line_word
                       WHERE ayah_id IS NOT NULL GROUP BY ayah_id) last
                   ON last.ayah_id = w.ayah_id AND last.mw = w.word_index
                 WHERE INSTR(w.glyph_v4, ' ') = 0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unmarked, 0, "all 6236 Ayahs end with a marker glyph");

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Tafsir is content, not schema, so migration 007's ALTERs leave an
    /// upgrading install with the right columns and nothing in them. This is
    /// the half that actually puts al-Jalalayn in front of an existing user.
    /// 007 delivers the *shape* tafsir needs and none of the content, because
    /// no edition ships with the app any more — every one of them is a pack the
    /// reader downloads. What used to be asserted here, that both bundled
    /// editions were copied out of the seed, describes a mechanism that no
    /// longer exists.
    #[test]
    fn upgrade_creates_the_tafsir_schema_and_installs_nothing() {
        let dir = std::env::temp_dir().join(format!("quranreader-tafsir-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);
        v011_install(&path);

        let conn = open(&path).unwrap();

        let editions: u32 = conn
            .query_row("SELECT COUNT(*) FROM tafsir", [], |r| r.get(0))
            .unwrap();
        assert_eq!(editions, 0, "no commentary ships in the seed");

        // These are the columns a pack install writes into. Their absence
        // would not surface until someone had already downloaded 25 MB.
        let has_column = |table: &str, column: &str| -> bool {
            conn.query_row(
                "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2",
                params![table, column],
                |_| Ok(()),
            )
            .optional()
            .unwrap()
            .is_some()
        };
        for column in [
            "slug",
            "translator",
            "name_native",
            "direction",
            "school",
            "creed",
            "source_url",
            "license",
            "sort_order",
        ] {
            assert!(has_column("tafsir", column), "tafsir.{column} is missing");
        }
        for column in ["group_start_ayah_id", "group_end_ayah_id"] {
            assert!(
                has_column("tafsir_ayah", column),
                "tafsir_ayah.{column} is missing — grouped editions need it"
            );
        }

        // The index is now kept in step by 007's triggers alone: the wholesale
        // copy that used to rebuild it went with the bundled editions, so a
        // downloaded pack's rows reach `fts_tafsir` through these or not at
        // all. Exercised with a row shaped like one a pack would insert.
        let ayah_id: i64 = conn
            .query_row(
                "SELECT id FROM ayah WHERE surah_id = 2 AND ayah_number = 255",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO tafsir
                (id, language, author, title, version, is_bundled, slug, direction, sort_order)
             VALUES (1000, 'en', 'Anon', 'Test edition', '1.0', 0, 'test-edition', 'ltr', 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tafsir_ayah (tafsir_id, ayah_id, text)
             VALUES (1000, ?1, 'a distinctive gloss')",
            params![ayah_id],
        )
        .unwrap();

        let indexed: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM fts_tafsir WHERE fts_tafsir MATCH 'distinctive'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(indexed, 1, "007's insert trigger must index a pack's rows");

        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The upgrade every current user takes: v0.1.9 shipped schema v6, so an
    /// installed copy is sitting at v6 with only 007 between it and this
    /// build. It is the shortest path in the table and the most travelled.
    ///
    /// Pinned because the numbering is the whole of it. Tafsir was written as
    /// a second migration 006 while reading history was still unreleased, and
    /// a second 006 would be skipped outright by every install that already
    /// recorded 6 — the tafsir schema would reach new installs and no one
    /// else, and a pack would then have nowhere to install itself.
    #[test]
    fn upgrade_from_v019_applies_007() {
        let dir = std::env::temp_dir().join(format!("quranreader-v019-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quran.db");
        let _ = std::fs::remove_file(&path);

        // A v0.1.9 install: the seed wound back past 007 only, so it keeps the
        // reading tables 006 gave it.
        std::fs::write(&path, SEED_DB).unwrap();
        let kursi_id: i64 = {
            let conn = Connection::open(&path).unwrap();
            undo_007(&conn);
            assert_eq!(
                get_schema_version(&conn).unwrap(),
                6,
                "fixture starts at v6"
            );

            let id: i64 = conn
                .query_row(
                    "SELECT id FROM ayah WHERE surah_id = 2 AND ayah_number = 255",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO reading_position (scope, scope_id, ayah_id)
                 VALUES ('juz', 3, ?1)",
                params![id],
            )
            .unwrap();
            id
        };

        let conn = open(&path).unwrap();

        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);

        // The tables arrive empty and stay that way: no edition ships with
        // the app, so what this upgrade delivers is the schema a downloaded
        // pack needs, not any commentary.
        let entries: u32 = conn
            .query_row("SELECT COUNT(*) FROM tafsir_ayah", [], |r| r.get(0))
            .unwrap();
        assert_eq!(entries, 0);
        let editions: u32 = conn
            .query_row("SELECT COUNT(*) FROM tafsir", [], |r| r.get(0))
            .unwrap();
        assert_eq!(editions, 0);

        // 006's user data is not disturbed on the way past. The seed ships one
        // position of its own — the 1:1 row 006 derives from the default
        // settings — so this install has that and the one it recorded itself.
        let ayah_id: i64 = conn
            .query_row(
                "SELECT ayah_id FROM reading_position WHERE scope = 'juz' AND scope_id = 3",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ayah_id, kursi_id, "the Juz-scoped position survives 007");

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
            undo_007(&conn);
            undo_006(&conn);
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
        assert_eq!(rows, 77_545);
        assert_eq!(null_v4, 0);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
