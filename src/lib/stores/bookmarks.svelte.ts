import { SvelteSet } from 'svelte/reactivity';
import { getBookmarks, toggleBookmark } from '$lib/api/db';

class BookmarksStore {
  ids = new SvelteSet<number>();
  loaded = $state(false);

  async init() {
    if (this.loaded) return;
    try {
      const list = await getBookmarks();
      list.forEach((b) => this.ids.add(b.ayah_id));
    } catch (err) {
      console.error('Failed to load bookmarks', err);
    } finally {
      this.loaded = true;
    }
  }

  isBookmarked(ayahId: number): boolean {
    return this.ids.has(ayahId);
  }

  async toggle(ayahId: number) {
    const nowBookmarked = await toggleBookmark(ayahId);
    if (nowBookmarked) {
      this.ids.add(ayahId);
    } else {
      this.ids.delete(ayahId);
    }
  }
}

export const bookmarksStore = new BookmarksStore();
