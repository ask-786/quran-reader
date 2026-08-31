//! Recitation audio: the reciter catalogue, the on-disk cache, and the fetcher
//! that fills it.
//!
//! Nothing here ships with the app and nothing here is published by it. An Ayah
//! of recitation is fetched from a public CDN the first time the reader plays
//! it, written under the app's data directory, and played from that file ever
//! after — so "offline audio" is what you have already listened to, rather than
//! a download step anyone had to plan in advance. See docs/audio-plan.md.
//!
//! # Why per-Ayah files
//!
//! The same host serves whole-Surah files, and al-Baqara's is 121 MB with no
//! Ayah boundaries in it. Per-Ayah files are 50–420 KB at 64 kbps, which is
//! small enough to fetch on a keypress, and they give Repeat Ayah and
//! follow-highlighting without a single millisecond of timing data.
//!
//! # What is checked, and what cannot be
//!
//! `packs` hashes every downloaded byte against a SHA-256 compiled into the
//! binary. That is not available here: the bytes come from a host that can
//! re-encode its library, and there is no published manifest to compile in.
//! What makes the weaker guarantee acceptable is not a matter of degree. A bad
//! pack writes wrong commentary into the reader's own database, silently and
//! permanently. Audio is never merged into the database and never executed —
//! it is a file the reader hears the moment it plays, and deleting it costs
//! nothing. So this checks what it can (host, redirects, content type, MP3
//! magic, size bounds) and leans on `.part` + rename so a truncated fetch is
//! never mistaken for a cached one.

use crate::db::error::{DbError, DbResult};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;

/// The only host this build will fetch from. Verse audio is addressed as
/// `{HOST}/quran/audio/{bitrate}/{reciter}/{ayah_id}.mp3`, where `ayah_id` is
/// the global 1–6236 Mushaf numbering — which is exactly `ayah.id` in this
/// database (`262.mp3` is 2:255 on both sides). That coincidence is why the
/// audio layer needs no mapping table.
const HOST: &str = "https://cdn.islamic.network";

/// Bitrates the host actually serves. 32, 48 and 192 all return 403, so there
/// is no low-bandwidth tier to offer however much one would help.
pub const BITRATES: &[u32] = &[64, 128];

/// Refuse anything too small to be a verse or too large to be one. The
/// shortest Ayah is a second of audio; the longest (2:282) is minutes. These
/// are sanity bounds, not a measurement — their job is to catch an error page
/// or a truncated body, not to police encoders.
const MIN_BYTES: u64 = 1_000;
const MAX_BYTES: u64 = 12 * 1024 * 1024;

/// Read size while streaming a file to disk.
const CHUNK: usize = 32 * 1024;

/// One whole-Quran fetch is 6,236 requests. Nothing here should be able to sit
/// on a half-open connection forever and stall the queue behind it.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("{0} kbps is not offered")]
    UnknownBitrate(u32),

    #[error("Download failed: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] DbError),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// What arrived is not audio. Nearly always an HTML error page or a
    /// captive portal's login screen saved under an `.mp3` name.
    #[error("The server did not return audio for this verse ({0})")]
    NotAudio(String),
}

pub type AudioResult<T> = Result<T, AudioError>;

/// One reciter this build can fetch.
///
/// Compiled in rather than fetched, for the reason `packs::PACKS` is: a list
/// pulled from a server would be a second document to trust, and one that
/// could point the app at any URL it liked. Adding a reciter is an app release.
///
/// **A reciter goes in this list only after every one of the 6,236 ids has
/// been checked.** `ar.minshawi` is a perfectly plausible identifier that
/// returns 403, and the failure a reader would see is the app stopping in the
/// middle of a Surah.
pub struct ReciterSpec {
    /// Doubles as the CDN's identifier and the cache directory name.
    pub slug: &'static str,
    pub name_ar: &'static str,
    pub name_en: &'static str,
    /// Which reading. Labelled for the same reason a tafsir carries its school
    /// and creed: it decides what you are hearing, and it is not visible in the
    /// audio itself.
    pub riwaya: &'static str,
    /// "murattal" | "mujawwad"
    pub style: &'static str,
    pub source_url: &'static str,
    pub license: &'static str,
    pub sort_order: u32,
}

/// Verified reachable at 64 kbps on 2026-08-31.
///
/// These are copyrighted recordings served by a third party. The app fetches
/// them at the reader's request and never hosts, repackages or redistributes
/// them — which is the reason this is a list of URLs rather than a list of
/// release assets.
pub const RECITERS: &[ReciterSpec] = &[
    ReciterSpec {
        slug: "ar.alafasy",
        name_ar: "مشاري راشد العفاسي",
        name_en: "Mishary Rashid Alafasy",
        riwaya: "Ḥafṣ ʿan ʿĀṣim",
        style: "murattal",
        source_url: "https://cdn.islamic.network/quran/audio",
        license: "© the reciter and publisher — streamed from the source, not redistributed",
        sort_order: 0,
    },
    ReciterSpec {
        slug: "ar.husary",
        name_ar: "محمود خليل الحصري",
        name_en: "Mahmoud Khalil Al-Husary",
        riwaya: "Ḥafṣ ʿan ʿĀṣim",
        style: "murattal",
        source_url: "https://cdn.islamic.network/quran/audio",
        license: "© the reciter and publisher — streamed from the source, not redistributed",
        sort_order: 1,
    },
    ReciterSpec {
        slug: "ar.abdulbasitmurattal",
        name_ar: "عبد الباسط عبد الصمد",
        name_en: "Abdul Basit Abdus Samad",
        riwaya: "Ḥafṣ ʿan ʿĀṣim",
        style: "murattal",
        source_url: "https://cdn.islamic.network/quran/audio",
        license: "© the reciter and publisher — streamed from the source, not redistributed",
        sort_order: 2,
    },
    ReciterSpec {
        slug: "ar.shaatree",
        name_ar: "أبو بكر الشاطري",
        name_en: "Abu Bakr Ash-Shaatree",
        riwaya: "Ḥafṣ ʿan ʿĀṣim",
        style: "murattal",
        source_url: "https://cdn.islamic.network/quran/audio",
        license: "© the reciter and publisher — streamed from the source, not redistributed",
        sort_order: 3,
    },
    ReciterSpec {
        slug: "ar.abdurrahmaansudais",
        name_ar: "عبد الرحمن السديس",
        name_en: "Abdur-Rahman As-Sudais",
        riwaya: "Ḥafṣ ʿan ʿĀṣim",
        style: "murattal",
        source_url: "https://cdn.islamic.network/quran/audio",
        license: "© the reciter and publisher — streamed from the source, not redistributed",
        sort_order: 4,
    },
];

/// A reciter as the frontend sees one: the catalogue entry plus the row id the
/// settings key stores.
#[derive(Debug, Clone, Serialize)]
pub struct Reciter {
    pub id: u32,
    pub slug: String,
    pub name_ar: String,
    pub name_en: String,
    pub riwaya: String,
    pub style: String,
    pub source_url: Option<String>,
    pub license: Option<String>,
}

/// What one reciter's audio costs on disk, for the Audio settings section.
#[derive(Debug, Clone, Serialize)]
pub struct ReciterUsage {
    pub reciter_id: u32,
    pub slug: String,
    pub name_en: String,
    pub files: u32,
    pub bytes: u64,
}

/// Set while the reader has asked to stop an eager download. A Juz is ~300
/// files and several minutes; a download that cannot be called off is a worse
/// feature than no download button at all.
static CANCEL_DOWNLOAD: AtomicBool = AtomicBool::new(false);

pub fn request_cancel() {
    CANCEL_DOWNLOAD.store(true, Ordering::Relaxed);
}

pub fn clear_cancel() {
    CANCEL_DOWNLOAD.store(false, Ordering::Relaxed);
}

pub fn cancelled() -> bool {
    CANCEL_DOWNLOAD.load(Ordering::Relaxed)
}

pub fn find(slug: &str) -> Option<&'static ReciterSpec> {
    RECITERS.iter().find(|r| r.slug == slug)
}

fn check_bitrate(bitrate: u32) -> AudioResult<()> {
    if BITRATES.contains(&bitrate) {
        Ok(())
    } else {
        Err(AudioError::UnknownBitrate(bitrate))
    }
}

// =============================================================================
// CATALOGUE
// =============================================================================

/// Write the compiled catalogue into `reciter`, keyed by slug.
///
/// Runs on every open. Upsert rather than insert-if-empty so a renamed reciter
/// or a corrected riwāya reaches an existing install without a migration, and
/// so `reciter.id` — which `settings.reciter_id` stores — survives it.
pub fn sync_catalog(conn: &Connection) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    for spec in RECITERS {
        tx.execute(
            "INSERT INTO reciter
                 (slug, name_ar, name_en, riwaya, style, source_url, license, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(slug) DO UPDATE SET
                 name_ar    = excluded.name_ar,
                 name_en    = excluded.name_en,
                 riwaya     = excluded.riwaya,
                 style      = excluded.style,
                 source_url = excluded.source_url,
                 license    = excluded.license,
                 sort_order = excluded.sort_order",
            params![
                spec.slug,
                spec.name_ar,
                spec.name_en,
                spec.riwaya,
                spec.style,
                spec.source_url,
                spec.license,
                spec.sort_order
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn list(conn: &Connection) -> DbResult<Vec<Reciter>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name_ar, name_en, riwaya, style, source_url, license
         FROM reciter ORDER BY sort_order, name_en",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Reciter {
            id: r.get(0)?,
            slug: r.get(1)?,
            name_ar: r.get(2)?,
            name_en: r.get(3)?,
            riwaya: r.get(4)?,
            style: r.get(5)?,
            source_url: r.get(6)?,
            license: r.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

// =============================================================================
// CACHE LAYOUT
// =============================================================================

/// Where cached audio lives: beside the database, under the app's data
/// directory, so one "clear audio" removes a directory the app owns outright.
pub fn root(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("audio")
}

/// `<root>/<reciter>/<bitrate>/<ayah_id>.mp3`.
///
/// Bitrate is a directory rather than part of the filename so that switching
/// bitrate cannot half-overwrite a cache, and so clearing one is a directory
/// removal.
pub fn cache_path(root: &Path, slug: &str, bitrate: u32, ayah_id: u32) -> PathBuf {
    root.join(slug)
        .join(bitrate.to_string())
        .join(format!("{ayah_id}.mp3"))
}

fn url(slug: &str, bitrate: u32, ayah_id: u32) -> String {
    format!("{HOST}/quran/audio/{bitrate}/{slug}/{ayah_id}.mp3")
}

/// Two prefetches for the same verse would otherwise write the same staging
/// file. The counter makes each attempt's staging name its own.
static STAGE_NONCE: AtomicU64 = AtomicU64::new(0);

fn staging_path(final_path: &Path) -> PathBuf {
    let nonce = STAGE_NONCE.fetch_add(1, Ordering::Relaxed);
    let mut name = final_path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".part{nonce}"));
    final_path.with_file_name(name)
}

// =============================================================================
// FETCHING
// =============================================================================

/// Does this look like an MP3 rather than someone's error page?
///
/// An ID3v2 tag, or an MPEG frame sync (11 set bits). Both hosts serve files
/// with an ID3 header, but the sync check keeps a stripped file playable.
fn looks_like_mp3(head: &[u8]) -> bool {
    if head.starts_with(b"ID3") {
        return true;
    }
    head.len() >= 2 && head[0] == 0xFF && (head[1] & 0xE0) == 0xE0
}

/// Fetch one Ayah to `dest`, or leave nothing behind.
///
/// Redirects are refused outright rather than followed and re-checked: this
/// build knows exactly one host and one URL shape, and a 3xx from it is a
/// change worth failing loudly on rather than chasing.
fn download(slug: &str, bitrate: u32, ayah_id: u32, dest: &Path) -> AudioResult<u64> {
    let agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .max_redirects_will_error(true)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .new_agent();

    let response = agent
        .get(url(slug, bitrate, ayah_id))
        .call()
        .map_err(|err| AudioError::Http(err.to_string()))?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !content_type.starts_with("audio/") {
        return Err(AudioError::NotAudio(if content_type.is_empty() {
            "no content type".to_string()
        } else {
            content_type
        }));
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let staged = staging_path(dest);
    let result = (|| -> AudioResult<u64> {
        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(&staged)?;
        let mut buf = vec![0u8; CHUNK];
        let mut written: u64 = 0;
        let mut head: Vec<u8> = Vec::new();

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            if head.len() < 4 {
                head.extend_from_slice(&buf[..n.min(4)]);
            }
            written += n as u64;
            if written > MAX_BYTES {
                return Err(AudioError::NotAudio(format!(
                    "over {} MB for one verse",
                    MAX_BYTES / 1_048_576
                )));
            }
            std::io::Write::write_all(&mut file, &buf[..n])?;
        }
        std::io::Write::flush(&mut file)?;
        drop(file);

        if written < MIN_BYTES {
            return Err(AudioError::NotAudio(format!("only {written} bytes")));
        }
        if !looks_like_mp3(&head) {
            return Err(AudioError::NotAudio("not an MP3".to_string()));
        }

        // Rename last, so a file at the final path is always a whole one. A
        // reader who pulls the network cable mid-fetch gets a missing verse,
        // never a clipped one.
        std::fs::rename(&staged, dest)?;
        Ok(written)
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

/// The cached file for one Ayah, or `None` if it is not on disk.
///
/// Filesystem only, and deliberately so: this is called before every fetch and
/// once per prefetch, and taking the database lock to answer "do I have this
/// file" would put a mutex between the reader and the next verse.
///
/// Anything too small to be a verse is treated as absent and removed. That is a
/// leftover from a build that failed mid-write, and playing it would be a click
/// of noise where an Ayah should be.
pub fn cached_path(root: &Path, slug: &str, bitrate: u32, ayah_id: u32) -> Option<PathBuf> {
    let path = cache_path(root, slug, bitrate, ayah_id);
    match std::fs::metadata(&path) {
        Ok(meta) if meta.len() >= MIN_BYTES => Some(path),
        Ok(_) => {
            let _ = std::fs::remove_file(&path);
            None
        }
        Err(_) => None,
    }
}

/// Fetch one Ayah into the cache. Network and filesystem only.
///
/// The database is untouched here on purpose. A fetch is the slow part, and
/// `packs::install_blocking` already established the rule this follows: the
/// lock is taken after the slow part is over, never across it, so a download
/// cannot block every other query for its duration. The caller records the
/// result with [`record`].
pub fn fetch_to_cache(
    root: &Path,
    spec: &ReciterSpec,
    bitrate: u32,
    ayah_id: u32,
) -> AudioResult<(PathBuf, u64)> {
    check_bitrate(bitrate)?;
    let path = cache_path(root, spec.slug, bitrate, ayah_id);
    let bytes = download(spec.slug, bitrate, ayah_id, &path)?;
    Ok((path, bytes))
}

/// Note a cached file in `audio_file`. Cheap, and takes the lock for the length
/// of one upsert.
pub fn record(
    conn: &Connection,
    reciter_id: u32,
    bitrate: u32,
    ayah_id: u32,
    bytes: u64,
) -> AudioResult<()> {
    conn.execute(
        "INSERT INTO audio_file (reciter_id, bitrate, ayah_id, bytes)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(reciter_id, bitrate, ayah_id) DO UPDATE SET bytes = excluded.bytes",
        params![reciter_id, bitrate, ayah_id, bytes as i64],
    )?;
    Ok(())
}

/// Which Ayahs of a range are already on disk — what the download button
/// counts before it offers to fetch the rest.
pub fn cached_in_range(
    conn: &Connection,
    reciter_id: u32,
    bitrate: u32,
    first_ayah_id: u32,
    last_ayah_id: u32,
) -> DbResult<Vec<u32>> {
    let mut stmt = conn.prepare(
        "SELECT ayah_id FROM audio_file
         WHERE reciter_id = ?1 AND bitrate = ?2 AND ayah_id BETWEEN ?3 AND ?4
         ORDER BY ayah_id",
    )?;
    let rows = stmt.query_map(
        params![reciter_id, bitrate, first_ayah_id, last_ayah_id],
        |r| r.get(0),
    )?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

// =============================================================================
// STORAGE
// =============================================================================

/// What each reciter's cache costs, newest figures first computed from
/// `audio_file` rather than the filesystem — see `reconcile` for what keeps
/// the two in step.
pub fn usage(conn: &Connection) -> DbResult<Vec<ReciterUsage>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.slug, r.name_en,
                COUNT(a.ayah_id), COALESCE(SUM(a.bytes), 0)
         FROM reciter r
         LEFT JOIN audio_file a ON a.reciter_id = r.id
         GROUP BY r.id
         ORDER BY r.sort_order, r.name_en",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ReciterUsage {
            reciter_id: r.get(0)?,
            slug: r.get(1)?,
            name_en: r.get(2)?,
            files: r.get(3)?,
            bytes: r.get::<_, i64>(4)? as u64,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Bring `audio_file` back in step with the directory it describes.
///
/// The files are the truth; the table is an index over them, and the two can
/// disagree after a restore from backup, a manual delete, or a crash between
/// the rename and the insert. Counting directory entries is cheap, so the
/// expensive part — a `stat` per file — only runs for a reciter and bitrate
/// whose counts already disagree.
pub fn reconcile(conn: &Connection, root: &Path) -> DbResult<()> {
    for spec in RECITERS {
        let reciter_id: Option<u32> = conn
            .query_row(
                "SELECT id FROM reciter WHERE slug = ?1",
                params![spec.slug],
                |r| r.get(0),
            )
            .ok();
        let Some(reciter_id) = reciter_id else {
            continue;
        };

        for &bitrate in BITRATES {
            let dir = root.join(spec.slug).join(bitrate.to_string());
            let on_disk: Vec<(u32, PathBuf)> = match std::fs::read_dir(&dir) {
                Ok(entries) => entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let path = e.path();
                        let stem = path.file_stem()?.to_str()?;
                        if path.extension()?.to_str()? != "mp3" {
                            return None;
                        }
                        Some((stem.parse::<u32>().ok()?, path))
                    })
                    .collect(),
                Err(_) => Vec::new(),
            };

            let recorded: u32 = conn.query_row(
                "SELECT COUNT(*) FROM audio_file WHERE reciter_id = ?1 AND bitrate = ?2",
                params![reciter_id, bitrate],
                |r| r.get(0),
            )?;

            if recorded as usize == on_disk.len() {
                continue;
            }

            log::info!(
                "Audio cache for {} at {} kbps: {} files on disk, {} recorded — reconciling",
                spec.slug,
                bitrate,
                on_disk.len(),
                recorded
            );

            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "DELETE FROM audio_file WHERE reciter_id = ?1 AND bitrate = ?2",
                params![reciter_id, bitrate],
            )?;
            {
                let mut insert = tx.prepare(
                    "INSERT INTO audio_file (reciter_id, bitrate, ayah_id, bytes)
                     VALUES (?1, ?2, ?3, ?4)",
                )?;
                for (ayah_id, path) in on_disk {
                    let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    if bytes < MIN_BYTES {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    insert.execute(params![reciter_id, bitrate, ayah_id, bytes as i64])?;
                }
            }
            tx.commit()?;
        }
    }
    Ok(())
}

/// Delete cached audio: one reciter's, or all of it.
///
/// The directory goes first. If the process dies between the two, the next
/// `reconcile` clears the rows, whereas the reverse order would leave files
/// nothing knows about.
pub fn clear(conn: &Connection, root: &Path, slug: Option<&str>) -> AudioResult<u64> {
    let freed: u64 = match slug {
        Some(slug) => conn.query_row(
            "SELECT COALESCE(SUM(a.bytes), 0) FROM audio_file a
             JOIN reciter r ON r.id = a.reciter_id
             WHERE r.slug = ?1",
            params![slug],
            |r| r.get::<_, i64>(0),
        )? as u64,
        None => conn.query_row("SELECT COALESCE(SUM(bytes), 0) FROM audio_file", [], |r| {
            r.get::<_, i64>(0)
        })? as u64,
    };

    match slug {
        Some(slug) => {
            let dir = root.join(slug);
            if dir.exists() {
                std::fs::remove_dir_all(&dir)?;
            }
            conn.execute(
                "DELETE FROM audio_file
                 WHERE reciter_id IN (SELECT id FROM reciter WHERE slug = ?1)",
                params![slug],
            )?;
        }
        None => {
            if root.exists() {
                std::fs::remove_dir_all(root)?;
            }
            conn.execute("DELETE FROM audio_file", [])?;
        }
    }

    Ok(freed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real, migrated database in a throwaway file — `reciter` and
    /// `audio_file` have foreign keys into `ayah`, so these cannot run against
    /// an empty schema.
    fn test_db(name: &str) -> (PathBuf, Connection) {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "quranreader-audio-{name}-{nonce}-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let conn = crate::db::connection::open(&path).unwrap();
        (path, conn)
    }

    #[test]
    fn mp3_magic_accepts_id3_and_frame_sync() {
        assert!(looks_like_mp3(b"ID3\x04"));
        assert!(looks_like_mp3(&[0xFF, 0xFB, 0x90, 0x00]));
    }

    #[test]
    fn mp3_magic_rejects_an_error_page() {
        assert!(!looks_like_mp3(b"<!DO"));
        assert!(!looks_like_mp3(b"{\"er"));
        assert!(!looks_like_mp3(&[]));
    }

    #[test]
    fn cache_path_separates_reciters_and_bitrates() {
        let root = Path::new("/data/audio");
        assert_eq!(
            cache_path(root, "ar.alafasy", 64, 262),
            Path::new("/data/audio/ar.alafasy/64/262.mp3")
        );
        assert_ne!(
            cache_path(root, "ar.alafasy", 64, 262),
            cache_path(root, "ar.alafasy", 128, 262)
        );
    }

    /// 2:255 is id 262 in this database and 262.mp3 on the CDN. If that ever
    /// stops being true, every verse plays the wrong audio.
    #[test]
    fn url_uses_the_global_ayah_id() {
        assert_eq!(
            url("ar.alafasy", 64, 262),
            "https://cdn.islamic.network/quran/audio/64/ar.alafasy/262.mp3"
        );
    }

    #[test]
    fn the_catalogue_is_written_once_however_often_it_runs() {
        let (path, conn) = test_db("catalogue");

        sync_catalog(&conn).unwrap();
        let first = list(&conn).unwrap();
        assert_eq!(first.len(), RECITERS.len());

        // Every launch runs this. A second pass must correct the rows in place,
        // not add a second copy of every reciter.
        sync_catalog(&conn).unwrap();
        let second = list(&conn).unwrap();
        assert_eq!(second.len(), RECITERS.len());
        assert_eq!(
            first[0].id, second[0].id,
            "ids survive, and settings hold one"
        );

        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn clearing_removes_the_files_and_the_rows_together() {
        let (path, conn) = test_db("clear");
        sync_catalog(&conn).unwrap();

        let root = path.parent().unwrap().join(format!(
            "audio-test-{}",
            path.file_stem().unwrap().to_string_lossy()
        ));
        let slug = RECITERS[0].slug;
        let reciter_id: u32 = conn
            .query_row(
                "SELECT id FROM reciter WHERE slug = ?1",
                params![slug],
                |r| r.get(0),
            )
            .unwrap();

        let file = cache_path(&root, slug, 64, 262);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, vec![0u8; 2048]).unwrap();
        record(&conn, reciter_id, 64, 262, 2048).unwrap();

        assert_eq!(usage(&conn).unwrap()[0].files, 1);

        let freed = clear(&conn, &root, Some(slug)).unwrap();
        assert_eq!(freed, 2048);
        assert!(!file.exists(), "the file is gone");
        assert_eq!(
            usage(&conn).unwrap()[0].files,
            0,
            "and so is the row that indexed it"
        );

        drop(conn);
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&path);
    }

    /// The table is an index over the directory, and the directory wins. A
    /// database restored from an older backup must not keep claiming files that
    /// are no longer there.
    #[test]
    fn reconcile_rebuilds_the_index_from_the_files() {
        let (path, conn) = test_db("reconcile");
        sync_catalog(&conn).unwrap();

        let root = path.parent().unwrap().join(format!(
            "audio-reconcile-{}",
            path.file_stem().unwrap().to_string_lossy()
        ));
        let slug = RECITERS[0].slug;
        let reciter_id: u32 = conn
            .query_row(
                "SELECT id FROM reciter WHERE slug = ?1",
                params![slug],
                |r| r.get(0),
            )
            .unwrap();

        // Two rows recorded, one file actually on disk.
        record(&conn, reciter_id, 64, 262, 4096).unwrap();
        record(&conn, reciter_id, 64, 263, 4096).unwrap();
        let kept = cache_path(&root, slug, 64, 262);
        std::fs::create_dir_all(kept.parent().unwrap()).unwrap();
        std::fs::write(&kept, vec![0u8; 4096]).unwrap();

        reconcile(&conn, &root).unwrap();

        let after = usage(&conn).unwrap();
        assert_eq!(after[0].files, 1);
        assert_eq!(after[0].bytes, 4096);
        assert!(cached_path(&root, slug, 64, 263).is_none());

        drop(conn);
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&path);
    }

    /// The one test that touches the network, so it is opt-in:
    /// `cargo test -- --ignored fetches_a_real_ayah`.
    ///
    /// It is here because everything the fetcher guards against is a property
    /// of a live host — the content type it sends, the redirect it might start
    /// sending, the bytes it actually returns. Al-Fatiha 1 is `ayah.id` 1 and
    /// ~49 KB at 64 kbps.
    #[test]
    #[ignore]
    fn fetches_a_real_ayah_from_the_cdn() {
        let dir = std::env::temp_dir().join(format!("quranreader-fetch-{}", std::process::id()));
        let dest = dir.join("1.mp3");
        let _ = std::fs::remove_file(&dest);

        let bytes = download(RECITERS[0].slug, 64, 1, &dest).expect("fetch failed");

        assert!(bytes > MIN_BYTES, "got {bytes} bytes");
        assert!(dest.exists(), "the file was renamed into place");
        assert_eq!(std::fs::metadata(&dest).unwrap().len(), bytes);

        let mut head = [0u8; 4];
        std::io::Read::read_exact(&mut std::fs::File::open(&dest).unwrap(), &mut head).unwrap();
        assert!(looks_like_mp3(&head), "not an mp3: {head:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn staging_names_do_not_collide() {
        let dest = Path::new("/data/audio/ar.alafasy/64/262.mp3");
        assert_ne!(staging_path(dest), staging_path(dest));
    }
}
