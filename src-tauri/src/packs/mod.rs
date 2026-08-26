//! Downloadable tafsir content packs.
//!
//! Some editions are too large to put in the installer — Ibn Kathir's two are
//! 34.5 MB of text between them, against an 18 MB seed database that is baked
//! into the binary with `include_bytes!`, so bundling them would be paid for
//! by every user in every release whether or not they wanted the edition. They
//! are published as release assets instead and fetched on request.
//!
//! A pack is a SQLite file with the same two tables the app stores editions in,
//! so installing one is an `INSERT ... SELECT` across an `ATTACH` rather than a
//! parser. `quran-importer --emit-pack` builds them; see `emit_pack` there for
//! the writing side of this contract.
//!
//! # What is trusted, and what is checked
//!
//! Nothing about a downloaded file is assumed. The bytes are hashed and
//! compared against a SHA-256 compiled into this binary before SQLite is
//! allowed near them, so a pack is only installed if it is byte-for-byte the
//! file that was published alongside this build. That is the reason the
//! manifest is a constant here rather than something fetched: there is no
//! second document to trust, and adding an edition is an app release.

use crate::db::error::{DbError, DbResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Pack layout this build knows how to read. A pack declaring anything else is
/// refused rather than guessed at.
const PACK_FORMAT: u32 = 1;

/// The first `tafsir.id` an installed pack may take.
///
/// Bundled editions are copied out of the seed positionally, ids and all (see
/// `copy_bundled_tafsir_from_seed`), so an upgrade re-inserts them at exactly
/// the ids the seed gave them. A pack that had been allowed to take one of
/// those ids would collide on the primary key and lose the user's edition to a
/// routine upgrade. Keeping pack ids above anything the seed will ever reach is
/// what makes the two allocations independent.
const PACK_ID_BASE: u32 = 1000;

/// How much of a pack to read at a time while downloading and hashing. Packs
/// are tens of megabytes; none of this is held in memory.
const CHUNK: usize = 64 * 1024;

#[derive(Debug, Error)]
pub enum PackError {
    #[error("No such pack: {0}")]
    Unknown(String),

    #[error("{0} is already installed")]
    AlreadyInstalled(String),

    #[error("Download failed: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] DbError),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// The bytes that arrived are not the bytes that were published. Could be a
    /// truncated download, a proxy rewriting the response, or a tampered file —
    /// this cannot tell which, and does not need to.
    #[error("This download does not match the published file and was discarded (expected {expected}, got {actual})")]
    Corrupt { expected: String, actual: String },

    #[error("This pack was built for a different version of the app (format {found}, expected {PACK_FORMAT})")]
    Format { found: String },

    #[error("This pack was built against a different Quran text ({found} ayahs, this database has {expected})")]
    AyahMismatch { found: i64, expected: i64 },

    #[error("The downloaded file is not a valid pack: {0}")]
    Malformed(String),
}

pub type PackResult<T> = Result<T, PackError>;

/// One publishable edition: where to get it, how to know it arrived intact, and
/// enough about it to list before it is downloaded.
///
/// The descriptive fields duplicate what the pack itself carries, deliberately:
/// the picker has to show the user what they are about to download while the
/// pack is still on a server, and possibly while they are offline.
pub struct PackSpec {
    /// Matches `tafsir.slug` — the same id the importer knows the edition by.
    pub slug: &'static str,
    pub title: &'static str,
    pub author: &'static str,
    /// BCP-47, for the "Arabic"/"English" label in the list.
    pub language: &'static str,
    pub license: &'static str,
    /// Size of the file at `url`, for the confirmation before a download.
    pub download_bytes: u64,
    /// Roughly what it adds to the database once installed.
    pub installed_bytes: u64,
    pub url: &'static str,
    /// Lowercase hex SHA-256 of the file at `url`, printed by
    /// `quran-importer --emit-pack` when the file is built.
    pub sha256: &'static str,
}

/// The editions this build can install.
///
/// Adding one is three steps and they have to happen in this order:
///
///   1. `cargo run -p quran-importer -- --emit-pack <slug>` — writes
///      `packs/<slug>-v<version>.qpack` and prints its SHA-256.
///   2. Upload that file to the `packs` release on GitHub.
///   3. Add the entry below with the asset's URL and the printed hash.
///
/// A wrong hash here does not install the wrong thing; it refuses to install
/// anything. That is the intended failure.
pub const PACKS: &[PackSpec] = &[];

/// A pack as the frontend sees it: the spec, plus whether it is already here.
#[derive(Debug, Clone, Serialize)]
pub struct PackStatus {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
    pub license: String,
    pub download_bytes: u64,
    pub installed_bytes: u64,
    pub installed: bool,
}

pub fn find(slug: &str) -> Option<&'static PackSpec> {
    PACKS.iter().find(|p| p.slug == slug)
}

/// Every publishable edition, marked with whether this database already has it.
pub fn list(conn: &Connection) -> DbResult<Vec<PackStatus>> {
    let mut stmt = conn.prepare("SELECT 1 FROM tafsir WHERE slug = ?1")?;

    PACKS
        .iter()
        .map(|p| {
            let installed = stmt
                .query_row(params![p.slug], |_| Ok(()))
                .optional()?
                .is_some();
            Ok(PackStatus {
                slug: p.slug.to_string(),
                title: p.title.to_string(),
                author: p.author.to_string(),
                language: p.language.to_string(),
                license: p.license.to_string(),
                download_bytes: p.download_bytes,
                installed_bytes: p.installed_bytes,
                installed,
            })
        })
        .collect()
}

/// Stream a pack to `dest`, hashing as it goes, and refuse it unless the hash
/// matches what this build expects.
///
/// `progress` is called with (bytes so far, total) often enough to drive a bar
/// and rarely enough not to flood the IPC bridge — once per chunk.
///
/// Verification happens here rather than at install time so that a bad file is
/// never handed to SQLite at all. The file is removed on any failure: a
/// half-downloaded pack left on disk is something a later run would have to
/// distinguish from a good one, and the cheapest way to never need to is to not
/// leave one.
pub fn download_verified(
    spec: &PackSpec,
    dest: &Path,
    mut progress: impl FnMut(u64, u64),
) -> PackResult<()> {
    use sha2::{Digest, Sha256};

    let result = (|| -> PackResult<()> {
        let response = ureq::get(spec.url)
            .call()
            .map_err(|err| PackError::Http(err.to_string()))?;

        let total = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(spec.download_bytes);

        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(dest)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; CHUNK];
        let mut seen: u64 = 0;

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            std::io::Write::write_all(&mut file, &buf[..n])?;
            hasher.update(&buf[..n]);
            seen += n as u64;
            progress(seen, total);
        }
        std::io::Write::flush(&mut file)?;

        let actual: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if actual != spec.sha256 {
            return Err(PackError::Corrupt {
                expected: spec.sha256.to_string(),
                actual,
            });
        }

        Ok(())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(dest);
    }
    result
}

/// Install an already-downloaded, already-verified pack into the database.
///
/// Everything happens in one transaction: an interrupted install leaves the
/// edition absent rather than half-present, which matters because "half a
/// tafsir" is indistinguishable from a grouped edition's ordinary gaps.
pub fn install(conn: &Connection, spec: &PackSpec, pack_path: &Path) -> PackResult<u32> {
    if find_installed_id(conn, spec.slug)?.is_some() {
        return Err(PackError::AlreadyInstalled(spec.title.to_string()));
    }

    let path = pack_path.to_string_lossy().to_string();
    conn.execute("ATTACH DATABASE ?1 AS pack", params![path])?;

    let result = (|| -> PackResult<u32> {
        verify_pack_contents(conn)?;

        // Above both the seed's ids and anything already installed, so two
        // packs installed in either order cannot land on the same id.
        let next_id: u32 = conn.query_row(
            "SELECT MAX(next) FROM (
                 SELECT ?1 AS next
                 UNION ALL
                 SELECT COALESCE(MAX(id), 0) + 1 FROM tafsir
             )",
            params![PACK_ID_BASE],
            |r| r.get(0),
        )?;

        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO tafsir
                (id, language, author, title, version, is_bundled, slug, translator,
                 name_native, direction, school, creed, source_url, license, sort_order)
             SELECT ?1, language, author, title, version, 0, slug, translator,
                    name_native, direction, school, creed, source_url, license, sort_order
             FROM pack.tafsir",
            params![next_id],
        )?;
        tx.execute(
            "INSERT INTO tafsir_ayah
                (tafsir_id, ayah_id, text, group_start_ayah_id, group_end_ayah_id)
             SELECT ?1, ayah_id, text, group_start_ayah_id, group_end_ayah_id
             FROM pack.tafsir_ayah",
            params![next_id],
        )?;
        tx.commit()?;

        Ok(next_id)
    })();

    // Detached whether or not the install worked — an attached pack file is a
    // handle on a file the caller is about to delete.
    let _ = conn.execute_batch("DETACH DATABASE pack;");
    result
}

/// Refuse a pack this build cannot read, or one built against a different Quran
/// text.
///
/// The ayah check is the important one. Every row in a pack is keyed by
/// `ayah.id`, which is only meaningful against the Mushaf it was built from;
/// installing across a mismatch would attach commentary to the wrong verses
/// silently, which is far worse than refusing.
fn verify_pack_contents(conn: &Connection) -> PackResult<()> {
    let meta = |key: &str| -> PackResult<Option<String>> {
        conn.query_row(
            "SELECT value FROM pack.pack_meta WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|err| PackError::Malformed(err.to_string()))
    };

    let format = meta("format")?
        .ok_or_else(|| PackError::Malformed("no format recorded".to_string()))?;
    if format != PACK_FORMAT.to_string() {
        return Err(PackError::Format { found: format });
    }

    let found: i64 = meta("ayah_count")?
        .ok_or_else(|| PackError::Malformed("no ayah count recorded".to_string()))?
        .parse()
        .map_err(|_| PackError::Malformed("ayah count is not a number".to_string()))?;
    let expected: i64 = conn.query_row("SELECT COUNT(*) FROM ayah", [], |r| r.get(0))?;
    if found != expected {
        return Err(PackError::AyahMismatch { found, expected });
    }

    Ok(())
}

/// Remove an installed edition and everything it wrote.
///
/// Refuses to touch a bundled one: those are the app's, restored by every
/// upgrade anyway, and "uninstall" would be a button that appears to work and
/// silently undoes itself.
pub fn remove(conn: &Connection, slug: &str) -> PackResult<()> {
    let id = find_installed_id(conn, slug)?.ok_or_else(|| PackError::Unknown(slug.to_string()))?;

    let bundled: bool = conn.query_row(
        "SELECT is_bundled FROM tafsir WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )?;
    if bundled {
        return Err(PackError::Malformed(
            "that edition ships with the app and cannot be removed".to_string(),
        ));
    }

    let tx = conn.unchecked_transaction()?;
    // Explicit rather than leaning on ON DELETE CASCADE: the cascade only fires
    // with foreign_keys ON, which is a per-connection pragma, and the rows this
    // leaves behind would be invisible orphans keyed to a reused id.
    tx.execute("DELETE FROM tafsir_ayah WHERE tafsir_id = ?1", params![id])?;
    tx.execute("DELETE FROM tafsir WHERE id = ?1", params![id])?;
    tx.commit()?;

    Ok(())
}

fn find_installed_id(conn: &Connection, slug: &str) -> PackResult<Option<u32>> {
    conn.query_row("SELECT id FROM tafsir WHERE slug = ?1", params![slug], |r| {
        r.get(0)
    })
    .optional()
    .map_err(Into::into)
}

/// Where a download is staged. Beside the database rather than in the system
/// temp directory, so the verified file and its destination are on one
/// filesystem and the install cannot fail on a full `/tmp`.
pub fn staging_path(db_path: &Path, slug: &str) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{slug}.qpack.part"))
}
