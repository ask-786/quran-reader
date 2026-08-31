use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

mod audio;
mod commands;
mod db;
mod models;
mod packs;

pub use db::error::{DbError, DbResult};

/// Global database connection wrapped in a Mutex for thread-safe Tauri state.
pub struct AppDb(pub Mutex<Connection>);

/// Where that database lives, kept alongside the connection because a pack
/// install needs the *path* and not the handle: it stages the download beside
/// the database so the verified file and its destination share a filesystem.
pub struct DbPath(pub PathBuf);

/// Root of the recitation cache, `<data dir>/audio`. Held as state for the same
/// reason as [`DbPath`]: the audio commands work in paths, and the webview is
/// handed those paths directly through the asset protocol.
pub struct AudioRoot(pub PathBuf);

/// The application name handed to [`ProjectDirs`], and with it the data
/// directory the database lives in.
///
/// Debug builds deliberately pick a *different* directory from release ones.
/// `db::connection::open` migrates a database in place and has no downgrade
/// path, so a single `tauri dev` run on a branch that bumps `CURRENT_VERSION`
/// would otherwise upgrade the installed release's database underneath it —
/// the older binary then opens a newer schema, sees a version it considers
/// current, and queries columns a migration may already have dropped. Bookmarks
/// and notes share that same file, so the blast radius is the user's own data.
const APP_DIR: &str = if cfg!(debug_assertions) {
    "QuranReader-dev"
} else {
    "QuranReader"
};

/// Resolve the path of the SQLite database this build should open.
///
/// `QURAN_READER_DB` overrides everything — needed because `debug_assertions`
/// alone does not separate a locally built *release* binary from the installed
/// one, and for throwaway databases when testing an upgrade path.
fn resolve_db_path(app: &tauri::AppHandle) -> PathBuf {
    if let Some(custom) = std::env::var_os("QURAN_READER_DB") {
        return PathBuf::from(custom);
    }

    // e.g. ~/.local/share/quranreader/quran.db on Linux (ProjectDirs lowercases
    // the application name there and ignores the qualifier and organization).
    if let Some(proj) = ProjectDirs::from("com", "quranreader", APP_DIR) {
        return proj.data_dir().join("quran.db");
    }

    // Fallback: whatever Tauri considers the app data dir.
    app.path()
        .app_data_dir()
        .expect("Failed to resolve app data dir")
        .join("quran.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = resolve_db_path(app.handle());

            log::info!("Opening database at: {:?}", db_path);

            let conn = db::connection::open(&db_path).expect("Failed to open database");

            // The catalogue is compiled in, so it is written on every open
            // rather than seeded once: a corrected riwaya or a reciter added in
            // this release reaches an existing install without a migration.
            if let Err(err) = audio::sync_catalog(&conn) {
                log::error!("Failed to sync the reciter catalogue: {err}");
            }

            let audio_root = audio::root(&db_path);
            // The files on disk are the truth about what is cached; the table
            // is an index over them. This puts the two back in step after a
            // restore from backup or a manual delete, and it is cheap when
            // they already agree.
            if let Err(err) = audio::reconcile(&conn, &audio_root) {
                log::error!("Failed to reconcile the audio cache: {err}");
            }

            app.manage(AppDb(Mutex::new(conn)));
            app.manage(DbPath(db_path));
            app.manage(AudioRoot(audio_root));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::db_stats,
            commands::get_all_surahs,
            commands::get_surah,
            commands::get_ayahs_for_surah,
            commands::get_ayahs_for_page,
            commands::get_ayahs_for_juz,
            commands::get_ayahs_for_hizb,
            commands::get_page,
            commands::get_pages,
            commands::search_arabic,
            commands::get_bookmarks,
            commands::toggle_bookmark,
            commands::get_notes_for_ayah,
            commands::upsert_note,
            commands::delete_note,
            commands::record_reading_position,
            commands::get_reading_position,
            commands::get_last_reading_position,
            commands::get_reading_history,
            commands::clear_reading_history,
            commands::load_settings,
            commands::get_setting,
            commands::set_setting,
            commands::get_translations,
            commands::get_tafsirs,
            commands::get_tafsir_for_ayah,
            commands::list_tafsir_packs,
            commands::install_tafsir_pack,
            commands::remove_tafsir_pack,
            commands::list_reciters,
            commands::ensure_ayah_audio,
            commands::read_ayah_audio,
            commands::prefetch_ayah_audio,
            commands::cached_audio_in_range,
            commands::download_audio_range,
            commands::cancel_audio_download,
            commands::audio_usage,
            commands::clear_audio_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the whole point of `APP_DIR`: a debug build must never resolve to
    /// the directory the installed release is using. Tests are debug builds, so
    /// `APP_DIR` here is whatever `tauri dev` gets.
    #[test]
    fn dev_builds_use_a_separate_data_dir_from_release() {
        let dev = ProjectDirs::from("com", "quranreader", APP_DIR).unwrap();
        let release = ProjectDirs::from("com", "quranreader", "QuranReader").unwrap();

        assert_eq!(APP_DIR, "QuranReader-dev");
        assert_ne!(dev.data_dir(), release.data_dir());
    }
}
