import type { Surah } from '$lib/types/database';

export const BISMILLAH_TEXT = 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ';

/**
 * Surah 1 stores the Bismillah as Ayah 1 itself (no separate header).
 * Surah 9 has no Bismillah at all. Every other Surah shows it as a header.
 */
export function shouldShowBismillahHeader(surah: Surah): boolean {
  return surah.has_bismillah && surah.id !== 1;
}
