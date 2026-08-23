use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

mod commands;
mod db;
mod models;

pub use db::error::{DbError, DbResult};

/// Global database connection wrapped in a Mutex for thread-safe Tauri state.
pub struct AppDb(pub Mutex<Connection>);

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

            app.manage(AppDb(Mutex::new(conn)));
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
