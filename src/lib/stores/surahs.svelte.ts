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
}

export const surahsStore = new SurahsStore();
