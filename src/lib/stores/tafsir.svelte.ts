import { getTafsirs, getTafsirForAyah, setSetting } from '$lib/api/db';
import type { Tafsir, TafsirEntry } from '$lib/types/database';
import { settingsStore } from './settings.svelte';
import { readerPosition } from './reader-position.svelte';

/** Drawer width bounds, in CSS px before app zoom. */
export const TAFSIR_MIN_WIDTH = 280;
export const TAFSIR_MAX_WIDTH = 720;

/** Shared with the drag handle, so a drag can't take the panel somewhere the
 *  stored width would then be clamped away from. */
export function clampTafsirWidth(px: number): number {
  return Math.round(Math.min(TAFSIR_MAX_WIDTH, Math.max(TAFSIR_MIN_WIDTH, px)));
}

/**
 * Entries kept in memory. A cap rather than an unbounded map because the panel
 * loads one entry per Ayah the reader passes, and a long session down a long
 * Surah would otherwise hold the whole edition.
 */
const CACHE_LIMIT = 200;

class TafsirStore {
  editions = $state<Tafsir[]>([]);
  entry = $state<TafsirEntry | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  ready = $state(false);

  /**
   * Ayah the panel is showing because it was asked to, rather than because the
   * reader is sitting on it. Cleared as soon as the reader moves somewhere
   * else, so an explicit open reads as "show me this one" and not as a mode
   * the user has to find their way out of.
   */
  #pinnedAyahId = $state<number | null>(null);
  /** Reader position at the moment of pinning — see `syncPosition`. */
  #positionAtPin: number | null = null;

  #cache = new Map<string, TafsirEntry | null>();
  /** Guards against an earlier request resolving after a later one. */
  #requestToken = 0;

  get open() {
    return settingsStore.current.show_tafsir;
  }

  get width() {
    return settingsStore.current.tafsir_panel_width;
  }

  /**
   * The chosen edition, falling back to the first installed one so the panel
   * has something to show before any choice has been made.
   */
  get active(): Tafsir | undefined {
    const id = settingsStore.current.tafsir_id;
    return this.editions.find((t) => t.id === id) ?? this.editions[0];
  }

  /** Which Ayah the panel is for: the pinned one, else wherever the reader is. */
  get targetAyahId(): number | null {
    return this.#pinnedAyahId ?? readerPosition.ayahId;
  }

  async init() {
    if (this.ready) return;
    try {
      this.editions = await getTafsirs();
    } catch (err) {
      console.error('Failed to load tafsir editions', err);
      this.editions = [];
    }
    this.ready = true;
  }

  toggle() {
    void this.setOpen(!this.open);
  }

  /** Open the panel on a specific Ayah — the per-Ayah action in the reader. */
  openForAyah(ayahId: number) {
    this.#pinnedAyahId = ayahId;
    this.#positionAtPin = readerPosition.ayahId;
    void this.setOpen(true);
  }

  async setOpen(open: boolean) {
    if (!open) this.#pinnedAyahId = null;
    settingsStore.current.show_tafsir = open;
    await setSetting('show_tafsir', String(open));
  }

  async setEdition(id: number) {
    settingsStore.current.tafsir_id = id;
    // Entries are keyed by edition, so the cache stays valid; only what is on
    // screen has to be replaced.
    this.entry = null;
    await setSetting('tafsir_id', String(id));
  }

  async setWidth(px: number) {
    const clamped = clampTafsirWidth(px);
    settingsStore.current.tafsir_panel_width = clamped;
    await setSetting('tafsir_panel_width', String(clamped));
  }

  /**
   * Release the pin once the reader has actually moved off the Ayah it was set
   * from. Called with each new reader position; comparing against the position
   * captured at pin time is what distinguishes "the user scrolled" from "the
   * position was already there when they clicked".
   */
  syncPosition(positionAyahId: number | null) {
    if (this.#pinnedAyahId === null) return;
    if (positionAyahId !== this.#positionAtPin) {
      this.#pinnedAyahId = null;
      this.#positionAtPin = null;
    }
  }

  /**
   * Load the entry for `ayahId` under the active edition. A null result is a
   * real answer — this edition says nothing about that verse — and is cached
   * as such so scrolling back over it doesn't re-ask.
   */
  async load(tafsirId: number, ayahId: number) {
    const key = `${tafsirId}:${ayahId}`;
    const cached = this.#cache.get(key);
    if (cached !== undefined) {
      this.entry = cached;
      this.error = null;
      this.loading = false;
      return;
    }

    const token = ++this.#requestToken;
    this.loading = true;
    this.error = null;
    try {
      const entry = await getTafsirForAyah(tafsirId, ayahId);
      if (token !== this.#requestToken) return;
      this.#remember(key, entry);
      this.entry = entry;
    } catch (err) {
      if (token !== this.#requestToken) return;
      console.error('Failed to load tafsir', err);
      this.error = 'Could not load this commentary.';
      this.entry = null;
    } finally {
      if (token === this.#requestToken) this.loading = false;
    }
  }

  #remember(key: string, entry: TafsirEntry | null) {
    if (this.#cache.size >= CACHE_LIMIT) {
      // Oldest insertion first — Map preserves that order.
      const oldest = this.#cache.keys().next();
      if (!oldest.done) this.#cache.delete(oldest.value);
    }
    this.#cache.set(key, entry);
  }
}

export const tafsirStore = new TafsirStore();
