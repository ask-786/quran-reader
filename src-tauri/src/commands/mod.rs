//! Tauri IPC command handlers.
//! Each function is thin: lock the DB, call the appropriate query, map the error.

use crate::db::{connection, queries};
use crate::models::*;
use crate::AppDb;
use tauri::State;

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
