//! Tauri IPC command handlers.
//! Each function is thin: lock the DB, call the appropriate query, map the error.

use crate::db::{connection, queries};
use crate::models::*;
use crate::{audio, packs, AppDb, AudioRoot, DbPath};
use serde::Serialize;
use tauri::{Emitter, Manager, State};

// Helper macro to lock the DB and convert the MutexGuard error.
macro_rules! db {
    ($state:expr) => {
        $state
            .0
            .lock()
            .map_err(|_| "DB mutex poisoned".to_string())?
    };
}

// Helper to convert DbError → String for Tauri's `Result<T, String>` convention.
fn e(err: crate::db::error::DbError) -> String {
    err.to_string()
}

// =============================================================================
// DATABASE COMMANDS
// =============================================================================

#[tauri::command]
pub fn db_stats(state: State<AppDb>) -> Result<connection::DbStats, String> {
    let conn = db!(state);
    connection::stats(&conn).map_err(e)
}

// =============================================================================
// SURAH COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_all_surahs(state: State<AppDb>) -> Result<Vec<Surah>, String> {
    let conn = db!(state);
    queries::get_all_surahs(&conn).map_err(e)
}

#[tauri::command]
pub fn get_surah(state: State<AppDb>, surah_id: u32) -> Result<Surah, String> {
    let conn = db!(state);
    queries::get_surah(&conn, surah_id).map_err(e)
}

// =============================================================================
// AYAH COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_ayahs_for_surah(state: State<AppDb>, surah_id: u32) -> Result<Vec<Ayah>, String> {
    let conn = db!(state);
    queries::get_ayahs_for_surah(&conn, surah_id).map_err(e)
}

#[tauri::command]
pub fn get_ayahs_for_page(state: State<AppDb>, page: u32) -> Result<Vec<Ayah>, String> {
    let conn = db!(state);
    queries::get_ayahs_for_page(&conn, page).map_err(e)
}

#[tauri::command]
pub fn get_ayahs_for_juz(state: State<AppDb>, juz: u32) -> Result<Vec<Ayah>, String> {
    let conn = db!(state);
    queries::get_ayahs_for_juz(&conn, juz).map_err(e)
}

#[tauri::command]
pub fn get_ayahs_for_hizb(state: State<AppDb>, hizb: u32) -> Result<Vec<Ayah>, String> {
    let conn = db!(state);
    queries::get_ayahs_for_hizb(&conn, hizb).map_err(e)
}

// =============================================================================
// MUSHAF PAGE COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_page(state: State<AppDb>, page: u32) -> Result<MushafPage, String> {
    let conn = db!(state);
    queries::get_page(&conn, page).map_err(e)
}

#[tauri::command]
pub fn get_pages(state: State<AppDb>, start: u32, end: u32) -> Result<Vec<MushafPage>, String> {
    let conn = db!(state);
    queries::get_pages(&conn, start, end).map_err(e)
}

// =============================================================================
// SEARCH COMMANDS
// =============================================================================

#[tauri::command]
pub fn search_arabic(
    state: State<AppDb>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let conn = db!(state);
    queries::search_arabic(&conn, &query, limit.unwrap_or(50)).map_err(e)
}

// =============================================================================
// BOOKMARK COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_bookmarks(state: State<AppDb>) -> Result<Vec<Bookmark>, String> {
    let conn = db!(state);
    queries::get_bookmarks(&conn).map_err(e)
}

#[tauri::command]
pub fn toggle_bookmark(
    state: State<AppDb>,
    ayah_id: u32,
    label: Option<String>,
) -> Result<bool, String> {
    let conn = db!(state);
    queries::toggle_bookmark(&conn, ayah_id, label.as_deref()).map_err(e)
}

// =============================================================================
// NOTES COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_notes_for_ayah(state: State<AppDb>, ayah_id: u32) -> Result<Vec<Note>, String> {
    let conn = db!(state);
    queries::get_notes_for_ayah(&conn, ayah_id).map_err(e)
}

#[tauri::command]
pub fn upsert_note(
    state: State<AppDb>,
    note_id: Option<u32>,
    ayah_id: u32,
    content: String,
) -> Result<u32, String> {
    let conn = db!(state);
    queries::upsert_note(&conn, note_id, ayah_id, &content).map_err(e)
}

#[tauri::command]
pub fn delete_note(state: State<AppDb>, note_id: u32) -> Result<(), String> {
    let conn = db!(state);
    queries::delete_note(&conn, note_id).map_err(e)
}

// =============================================================================
// READING POSITION / HISTORY COMMANDS
// =============================================================================

#[tauri::command]
pub fn record_reading_position(
    state: State<AppDb>,
    scope: ReadingScope,
    scope_id: u32,
    ayah_id: u32,
) -> Result<(), String> {
    let conn = db!(state);
    queries::record_reading_position(&conn, scope, scope_id, ayah_id).map_err(e)
}

#[tauri::command]
pub fn get_reading_position(
    state: State<AppDb>,
    scope: ReadingScope,
    scope_id: u32,
) -> Result<Option<ReadingPosition>, String> {
    let conn = db!(state);
    queries::get_reading_position(&conn, scope, scope_id).map_err(e)
}

#[tauri::command]
pub fn get_last_reading_position(state: State<AppDb>) -> Result<Option<ReadingPosition>, String> {
    let conn = db!(state);
    queries::get_last_reading_position(&conn).map_err(e)
}

#[tauri::command]
pub fn get_reading_history(
    state: State<AppDb>,
    limit: Option<u32>,
) -> Result<Vec<ReadingSession>, String> {
    let conn = db!(state);
    queries::get_reading_history(&conn, limit.unwrap_or(50)).map_err(e)
}

#[tauri::command]
pub fn clear_reading_history(state: State<AppDb>) -> Result<(), String> {
    let conn = db!(state);
    queries::clear_reading_history(&conn).map_err(e)
}

// =============================================================================
// SETTINGS COMMANDS
// =============================================================================

#[tauri::command]
pub fn load_settings(state: State<AppDb>) -> Result<Settings, String> {
    let conn = db!(state);
    queries::load_settings(&conn).map_err(e)
}

#[tauri::command]
pub fn get_setting(state: State<AppDb>, key: String) -> Result<String, String> {
    let conn = db!(state);
    queries::get_setting(&conn, &key).map_err(e)
}

#[tauri::command]
pub fn set_setting(state: State<AppDb>, key: String, value: String) -> Result<(), String> {
    let conn = db!(state);
    queries::set_setting(&conn, &key, &value).map_err(e)
}

// =============================================================================
// TRANSLATION COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_translations(state: State<AppDb>) -> Result<Vec<Translation>, String> {
    let conn = db!(state);
    queries::get_translations(&conn).map_err(e)
}

// =============================================================================
// TAFSIR COMMANDS
// =============================================================================

#[tauri::command]
pub fn get_tafsirs(state: State<AppDb>) -> Result<Vec<Tafsir>, String> {
    let conn = db!(state);
    queries::get_tafsirs(&conn).map_err(e)
}

/// `Ok(None)` means this edition has no comment on that Ayah — an ordinary
/// result, not a failure. See `queries::get_tafsir_for_ayah`.
#[tauri::command]
pub fn get_tafsir_for_ayah(
    state: State<AppDb>,
    tafsir_id: u32,
    ayah_id: u32,
) -> Result<Option<TafsirEntry>, String> {
    let conn = db!(state);
    queries::get_tafsir_for_ayah(&conn, tafsir_id, ayah_id).map_err(e)
}

// =============================================================================
// TAFSIR PACK COMMANDS
// =============================================================================

/// Progress of a running download, emitted as `tafsir-pack-progress`.
#[derive(Clone, Serialize)]
pub struct PackProgress {
    pub slug: String,
    pub received: u64,
    pub total: u64,
}

#[tauri::command]
pub fn list_tafsir_packs(state: State<AppDb>) -> Result<Vec<packs::PackStatus>, String> {
    let conn = db!(state);
    packs::list(&conn).map_err(e)
}

/// Download, verify and install one edition. Returns its new `tafsir.id`.
///
/// `async` so Tauri runs it off the UI thread, and the body then goes straight
/// to `spawn_blocking`: the download is synchronous and tens of megabytes, so
/// leaving it on an async worker would tie that worker up for its duration.
#[tauri::command]
pub async fn install_tafsir_pack(app: tauri::AppHandle, slug: String) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || install_blocking(&app, &slug))
        .await
        .map_err(|err| err.to_string())?
}

fn install_blocking(app: &tauri::AppHandle, slug: &str) -> Result<u32, String> {
    let spec = packs::find(slug).ok_or_else(|| format!("No such pack: {slug}"))?;

    let db_path = app.state::<DbPath>().0.clone();
    let staged = packs::staging_path(&db_path, slug);

    // Throttled to whole percent: a 25 MB pack is ~400 chunks, and a repaint
    // per chunk buys the user nothing but a busier bridge.
    let mut last_percent = u64::MAX;
    let progress = |received: u64, total: u64| {
        // checked_div rather than a guard, so a response with no
        // content-length reports 0% instead of dividing by zero.
        let percent = (received * 100).checked_div(total).unwrap_or(0);
        if percent == last_percent {
            return;
        }
        last_percent = percent;
        let _ = app.emit(
            "tafsir-pack-progress",
            PackProgress {
                slug: spec.slug.to_string(),
                received,
                total,
            },
        );
    };

    packs::download_verified(spec, &staged, progress).map_err(|err| err.to_string())?;

    // The lock is taken only now, after the slow part is over, so a download
    // cannot block every other query for its duration.
    let result = {
        let state = app.state::<AppDb>();
        let conn = state
            .0
            .lock()
            .map_err(|_| "DB mutex poisoned".to_string())?;
        packs::install(&conn, spec, &staged).map_err(|err| err.to_string())
    };

    // The verified copy has served its purpose either way; the edition now
    // lives in the database.
    let _ = std::fs::remove_file(&staged);

    result
}

#[tauri::command]
pub fn remove_tafsir_pack(state: State<AppDb>, slug: String) -> Result<(), String> {
    let conn = db!(state);
    packs::remove(&conn, &slug).map_err(|err| err.to_string())
}

// =============================================================================
// AUDIO COMMANDS
// =============================================================================

/// Progress of a running range download, emitted as `audio-download-progress`.
#[derive(Clone, Serialize)]
pub struct AudioProgress {
    pub slug: String,
    /// Ayahs fetched or already present so far, out of `total`.
    pub done: u32,
    pub total: u32,
    /// The Ayah just finished, so the UI can show where it has reached.
    pub ayah_id: u32,
    /// Ayahs this run failed on. The run continues past them: one missing verse
    /// should not abandon the other 299.
    pub failed: u32,
}

/// Is fetching allowed right now?
///
/// Read from the database rather than taken as an argument. This is the switch
/// that decides whether the app touches the network at all, and the answer
/// should not depend on a frontend remembering to pass it.
fn downloads_allowed(state: &State<AppDb>) -> Result<bool, String> {
    let conn = db!(state);
    queries::get_setting(&conn, "audio_downloads_allowed")
        .map(|v| v == "true")
        .map_err(e)
}

fn reciter_id_for(state: &State<AppDb>, slug: &str) -> Result<u32, String> {
    let conn = db!(state);
    conn.query_row(
        "SELECT id FROM reciter WHERE slug = ?1",
        rusqlite::params![slug],
        |r| r.get(0),
    )
    .map_err(|_| format!("No such reciter: {slug}"))
}

#[tauri::command]
pub fn list_reciters(state: State<AppDb>) -> Result<Vec<audio::Reciter>, String> {
    let conn = db!(state);
    audio::list(&conn).map_err(e)
}

/// Make sure one Ayah is in the cache, fetching it if it is not.
///
/// Returns whether it is there now. `false` means the verse is not downloaded
/// and downloads are turned off — an answer to act on (the UI offers to turn
/// them on), not a failure. An error is a fetch that was tried and did not work.
#[tauri::command]
pub async fn ensure_ayah_audio(
    app: tauri::AppHandle,
    slug: String,
    bitrate: u32,
    ayah_id: u32,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || resolve_audio(&app, &slug, bitrate, ayah_id, true))
        .await
        .map_err(|err| err.to_string())?
        .map(|path| path.is_some())
}

/// The bytes of one cached Ayah, for the webview to play from a blob.
///
/// Handing the file over the IPC bridge rather than serving it through the
/// asset protocol, and that is not a preference. `convertFileSrc` yields
/// `asset://localhost/…` on Linux, and WebKitGTK's media pipeline cannot load
/// from a custom URI scheme — `fetch` and `<img>` can, `<audio>` cannot, and
/// the element fails with "The operation is not supported". A blob is decoded
/// in the page itself and has no scheme to reject. One verse is 50–420 KB, so
/// the copy costs nothing worth measuring.
///
/// Fetches first if the verse is not cached, so a caller needs one call rather
/// than two.
#[tauri::command]
pub async fn read_ayah_audio(
    app: tauri::AppHandle,
    slug: String,
    bitrate: u32,
    ayah_id: u32,
) -> Result<tauri::ipc::Response, String> {
    let bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let path = resolve_audio(&app, &slug, bitrate, ayah_id, true)?
            .ok_or_else(|| "This verse has not been downloaded".to_string())?;
        std::fs::read(&path).map_err(|err| err.to_string())
    })
    .await
    .map_err(|err| err.to_string())??;

    Ok(tauri::ipc::Response::new(bytes))
}

/// Pull an Ayah into the cache ahead of the playhead. Returns whether it is
/// there now.
///
/// Never reports an error. A prefetch that fails is a verse the player will ask
/// for again in a moment, on its own terms — surfacing it would put a network
/// error on screen for something the reader never asked for.
#[tauri::command]
pub async fn prefetch_ayah_audio(
    app: tauri::AppHandle,
    slug: String,
    bitrate: u32,
    ayah_id: u32,
) -> Result<bool, String> {
    let ready = tauri::async_runtime::spawn_blocking(move || {
        match resolve_audio(&app, &slug, bitrate, ayah_id, false) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(err) => {
                log::debug!("Prefetch of ayah {ayah_id} failed: {err}");
                false
            }
        }
    })
    .await
    .map_err(|err| err.to_string())?;
    Ok(ready)
}

/// Shared body of the three commands above: cache hit, or fetch and record.
///
/// `record_hit` is off for prefetches of files that were already cached: the
/// row is already right, and a prefetch pass over a downloaded Surah would
/// otherwise take the database lock once per verse for no change at all.
fn resolve_audio(
    app: &tauri::AppHandle,
    slug: &str,
    bitrate: u32,
    ayah_id: u32,
    record_hit: bool,
) -> Result<Option<std::path::PathBuf>, String> {
    let spec = audio::find(slug).ok_or_else(|| format!("No such reciter: {slug}"))?;
    let root = app.state::<AudioRoot>().0.clone();
    let state = app.state::<AppDb>();

    if let Some(path) = audio::cached_path(&root, slug, bitrate, ayah_id) {
        if record_hit {
            // Puts back a row lost with the database while the files survived
            // — a restore from backup, or a cache that outlived a reset.
            if let Ok(bytes) = std::fs::metadata(&path).map(|m| m.len()) {
                if let Ok(reciter_id) = reciter_id_for(&state, slug) {
                    let conn = db!(state);
                    let _ = audio::record(&conn, reciter_id, bitrate, ayah_id, bytes);
                }
            }
        }
        return Ok(Some(path));
    }

    if !downloads_allowed(&state)? {
        return Ok(None);
    }

    let reciter_id = reciter_id_for(&state, slug)?;
    // No lock held across this: it is the slow part.
    let (path, bytes) =
        audio::fetch_to_cache(&root, spec, bitrate, ayah_id).map_err(|err| err.to_string())?;

    {
        let conn = db!(state);
        audio::record(&conn, reciter_id, bitrate, ayah_id, bytes).map_err(|err| err.to_string())?;
    }

    Ok(Some(path))
}

/// Which Ayahs of an inclusive range are already cached — what the download
/// button counts before offering to fetch the rest.
#[tauri::command]
pub fn cached_audio_in_range(
    state: State<AppDb>,
    slug: String,
    bitrate: u32,
    first_ayah_id: u32,
    last_ayah_id: u32,
) -> Result<Vec<u32>, String> {
    let reciter_id = reciter_id_for(&state, &slug)?;
    let conn = db!(state);
    audio::cached_in_range(&conn, reciter_id, bitrate, first_ayah_id, last_ayah_id).map_err(e)
}

/// Fetch a whole range up front — a Surah or a Juz before a flight.
///
/// One verse at a time, deliberately. This is someone else's CDN serving files
/// the project does not host, and a parallel sweep over 6,236 objects is not a
/// reasonable thing to point at it. A Juz is ~300 files of ~150 KB and takes a
/// couple of minutes; `cancel_audio_download` stops it.
///
/// A verse that fails is counted and skipped. Abandoning the other 299 because
/// one object 404s would be the wrong trade for a reader packing for a journey.
#[tauri::command]
pub async fn download_audio_range(
    app: tauri::AppHandle,
    slug: String,
    bitrate: u32,
    first_ayah_id: u32,
    last_ayah_id: u32,
) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || {
        download_range_blocking(&app, &slug, bitrate, first_ayah_id, last_ayah_id)
    })
    .await
    .map_err(|err| err.to_string())?
}

fn download_range_blocking(
    app: &tauri::AppHandle,
    slug: &str,
    bitrate: u32,
    first_ayah_id: u32,
    last_ayah_id: u32,
) -> Result<u32, String> {
    let spec = audio::find(slug).ok_or_else(|| format!("No such reciter: {slug}"))?;
    let root = app.state::<AudioRoot>().0.clone();
    let state = app.state::<AppDb>();

    if !downloads_allowed(&state)? {
        return Err("Downloads are turned off in Settings → Audio".to_string());
    }
    let reciter_id = reciter_id_for(&state, slug)?;

    audio::clear_cancel();
    let total = last_ayah_id.saturating_sub(first_ayah_id) + 1;
    let mut done = 0u32;
    let mut failed = 0u32;

    for ayah_id in first_ayah_id..=last_ayah_id {
        if audio::cancelled() {
            break;
        }

        if audio::cached_path(&root, slug, bitrate, ayah_id).is_none() {
            match audio::fetch_to_cache(&root, spec, bitrate, ayah_id) {
                Ok((_, bytes)) => {
                    let conn = db!(state);
                    let _ = audio::record(&conn, reciter_id, bitrate, ayah_id, bytes);
                }
                Err(err) => {
                    log::warn!("Audio download failed for ayah {ayah_id}: {err}");
                    failed += 1;
                }
            }
        }

        done += 1;
        let _ = app.emit(
            "audio-download-progress",
            AudioProgress {
                slug: slug.to_string(),
                done,
                total,
                ayah_id,
                failed,
            },
        );
    }

    audio::clear_cancel();
    Ok(done.saturating_sub(failed))
}

/// Ask a running range download to stop. It finishes the verse in flight and
/// returns what it managed.
#[tauri::command]
pub fn cancel_audio_download() {
    audio::request_cancel();
}

/// What each reciter's cached audio costs on disk.
#[tauri::command]
pub fn audio_usage(state: State<AppDb>) -> Result<Vec<audio::ReciterUsage>, String> {
    let conn = db!(state);
    audio::usage(&conn).map_err(e)
}

/// Delete cached audio — one reciter's, or every reciter's. Returns the bytes
/// freed.
///
/// Nothing else is touched: the reciter stays in the catalogue, the chosen
/// reciter stays chosen, and playing a verse downloads it again.
#[tauri::command]
pub fn clear_audio_cache(
    state: State<AppDb>,
    root: State<AudioRoot>,
    slug: Option<String>,
) -> Result<u64, String> {
    let conn = db!(state);
    audio::clear(&conn, &root.0, slug.as_deref()).map_err(|err| err.to_string())
}
