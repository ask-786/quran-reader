/**
 * Typed wrappers around Tauri `invoke()` for every backend command.
 * Import from here instead of calling invoke() directly in components.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Surah,
  Ayah,
  Translation,
  Tafsir,
  TafsirEntry,
  Bookmark,
  Note,
  Settings,
  SearchResult,
  DbStats,
  MushafPage,
} from '$lib/types/database';

// =============================================================================
// DATABASE
// =============================================================================

export function dbStats(): Promise<DbStats> {
  return invoke('db_stats');
}

// =============================================================================
// SURAH
// =============================================================================

export function getAllSurahs(): Promise<Surah[]> {
  return invoke('get_all_surahs');
}

export function getSurah(surahId: number): Promise<Surah> {
  return invoke('get_surah', { surahId });
}

// =============================================================================
// AYAH
// =============================================================================

export function getAyahsForSurah(surahId: number): Promise<Ayah[]> {
  return invoke('get_ayahs_for_surah', { surahId });
}

export function getAyahsForPage(page: number): Promise<Ayah[]> {
  return invoke('get_ayahs_for_page', { page });
}

export function getAyahsForJuz(juz: number): Promise<Ayah[]> {
  return invoke('get_ayahs_for_juz', { juz });
}

export function getAyahsForHizb(hizb: number): Promise<Ayah[]> {
  return invoke('get_ayahs_for_hizb', { hizb });
}

// =============================================================================
// MUSHAF PAGE
// =============================================================================

export function getPage(page: number): Promise<MushafPage> {
  return invoke('get_page', { page });
}

/** Inclusive page range in a single call. Pages with no layout rows are omitted. */
export function getPages(start: number, end: number): Promise<MushafPage[]> {
  return invoke('get_pages', { start, end });
}

// =============================================================================
// SEARCH
// =============================================================================

export function searchArabic(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke('search_arabic', { query, limit });
}

// =============================================================================
// BOOKMARKS
// =============================================================================

export function getBookmarks(): Promise<Bookmark[]> {
  return invoke('get_bookmarks');
}

/** Returns true if bookmark now exists, false if it was removed. */
export function toggleBookmark(ayahId: number, label?: string): Promise<boolean> {
  return invoke('toggle_bookmark', { ayahId, label: label ?? null });
}

// =============================================================================
// NOTES
// =============================================================================

export function getNotesForAyah(ayahId: number): Promise<Note[]> {
  return invoke('get_notes_for_ayah', { ayahId });
}

/**
 * Create a new note (noteId = undefined) or update an existing one.
 * Returns the note id.
 */
export function upsertNote(ayahId: number, content: string, noteId?: number): Promise<number> {
  return invoke('upsert_note', { noteId: noteId ?? null, ayahId, content });
}

export function deleteNote(noteId: number): Promise<void> {
  return invoke('delete_note', { noteId });
}

// =============================================================================
// SETTINGS
// =============================================================================

export function loadSettings(): Promise<Settings> {
  return invoke('load_settings');
}

export function getSetting(key: string): Promise<string> {
  return invoke('get_setting', { key });
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value });
}

// =============================================================================
// TRANSLATIONS
// =============================================================================

export function getTranslations(): Promise<Translation[]> {
  return invoke('get_translations');
}

// =============================================================================
// TAFSIR
// =============================================================================

/** Installed editions, bundled first. */
export function getTafsirs(): Promise<Tafsir[]> {
  return invoke('get_tafsirs');
}

/**
 * Commentary on one Ayah, or null where this edition passes over it — an
 * ordinary outcome, not an error. Al-Jalalayn's Arabic glosses 6,010 of the
 * 6,236 Ayahs; the panel shows the gap rather than an empty box.
 */
export function getTafsirForAyah(tafsirId: number, ayahId: number): Promise<TafsirEntry | null> {
  return invoke('get_tafsir_for_ayah', { tafsirId, ayahId });
}
