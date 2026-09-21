import { getAllSurahs } from '$lib/api/db';
import type { Surah } from '$lib/types/database';

class SurahsStore {
  list = $state<Surah[]>([]);
  loading = $state(true);
  error = $state<string | null>(null);

  async init() {
    if (this.list.length > 0) return;
    try {
      this.list = await getAllSurahs();
    } catch (err) {
      this.error = String(err);
      console.error('Failed to load surahs', err);
    } finally {
      this.loading = false;
    }
  }

  get(id: number): Surah | undefined {
    return this.list.find((s) => s.id === id);
  }

  /**
   * First and last Ayah id of the Surah `ayahId` belongs to. Ayah ids number
   * the whole Mushaf in order, so a Surah is the run after the verses of every
   * Surah before it. Undefined until the list has loaded.
   */
  rangeOf(ayahId: number): { first: number; last: number } | undefined {
    let first = 1;
    for (const surah of this.list) {
      const last = first + surah.verses_count - 1;
      if (ayahId <= last) return ayahId >= first ? { first, last } : undefined;
      first = last + 1;
    }
    return undefined;
  }
}

export const surahsStore = new SurahsStore();
