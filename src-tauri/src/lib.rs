use directories::ProjectDirs;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

mod commands;
mod db;
mod models;

pub use db::error::{DbError, DbResult};

/// Global database connection wrapped in a Mutex for thread-safe Tauri state.
pub struct AppDb(pub Mutex<Connection>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve OS-appropriate app data directory
            // e.g. ~/.local/share/com.quranreader.app/quran.db on Linux
            let db_path = if let Some(proj) = ProjectDirs::from("com", "quranreader", "QuranReader")
            {
                let data_dir = proj.data_dir().to_path_buf();
                data_dir.join("quran.db")
            } else {
                // Fallback: store next to the binary
                app.path()
                    .app_data_dir()
                    .expect("Failed to resolve app data dir")
                    .join("quran.db")
            };

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
            commands::load_settings,
            commands::get_setting,
            commands::set_setting,
            commands::get_translations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
