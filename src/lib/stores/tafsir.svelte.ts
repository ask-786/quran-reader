import { getTafsirs, getTafsirForAyah, setSetting } from '$lib/api/db';
import type { Tafsir, TafsirEntry, TafsirView } from '$lib/types/database';
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

export interface TafsirSelection {
  ayahId: number;
  /**
   * The element the popover points at. Held so it can be repositioned as the
   * reader scrolls — and so focus can be returned to whatever opened it.
   *
   * Both views can take this element out of the DOM while the popover is open
   * (`AyahRow` drops its contents outside the render window; `PageView` empties
   * lines outside the glyph window), so nothing may assume it is still
   * connected. See the freeze behaviour in TafsirPopover.
   */
  anchor: HTMLElement | null;
}

class TafsirStore {
  editions = $state<Tafsir[]>([]);
  entry = $state<TafsirEntry | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  ready = $state(false);

  /**
   * The Ayah the popover was opened for, and the element it is anchored to.
   *
   * This is an explicit choice by the reader and nothing clears it but a
   * dismissal — no scroll, no view switch. That is the whole difference from
   * the panel, which follows `readerPosition` and therefore decides for
   * itself what you are reading.
   *
   * Null when the popover is closed. Popover state is session-only: a
   * transient answer to a click is not worth restoring on launch.
   */
  selection = $state<TafsirSelection | null>(null);

  #cache = new Map<string, TafsirEntry | null>();
  /** Guards against an earlier request resolving after a later one. */
  #requestToken = 0;

  /** Whether the side panel is open. Unrelated to the popover. */
  get panelOpen() {
    return settingsStore.current.show_tafsir;
  }

  get view(): TafsirView {
    return settingsStore.current.tafsir_view;
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

  /**
   * Which Ayah is on show. The popover's is chosen; the panel's is wherever the
   * reader is, which is the character of each surface rather than an accident.
   */
  get targetAyahId(): number | null {
    if (this.view === 'popover') return this.selection?.ayahId ?? null;
    return readerPosition.ayahId;
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

  /**
   * Show commentary for one Ayah. In popover mode that anchors a popover to
   * `anchor`; in panel mode it opens the panel, which then follows the reader
   * as it always has.
   */
  openFor(ayahId: number, anchor: HTMLElement | null = null) {
    if (this.view === 'panel') {
      void this.setPanelOpen(true);
      return;
    }
    this.selection = { ayahId, anchor };
  }

  closePopover() {
    this.selection = null;
  }

  /**
   * What the toolbar button and `t` do: in popover mode, open on wherever the
   * reader is (the only defensible target when nothing was clicked) or close
   * an open one; in panel mode, toggle the panel.
   */
  toggleForReaderPosition() {
    if (this.view === 'panel') {
      void this.setPanelOpen(!this.panelOpen);
      return;
    }
    if (this.selection) {
      this.closePopover();
      return;
    }
    const ayahId = readerPosition.ayahId;
    if (ayahId === null) return;
    // Anchor to the Ayah's own element when the reader has one rendered; the
    // popover falls back to a viewport-centred position when it does not.
    const anchor = document.querySelector<HTMLElement>(`[data-ayah-id="${ayahId}"]`);
    this.selection = { ayahId, anchor };
  }

  async setPanelOpen(open: boolean) {
    settingsStore.current.show_tafsir = open;
    await setSetting('show_tafsir', String(open));
  }

  /**
   * Switch surface. Moving to the panel carries the popover's Ayah over by
   * opening the panel, so "expand this" lands on the same commentary rather
   * than on wherever the reader happens to be sitting.
   */
  async setView(view: TafsirView) {
    const carried = this.selection;
    settingsStore.current.tafsir_view = view;
    this.selection = null;
    if (view === 'panel') await this.setPanelOpen(true);
    else if (carried) this.selection = carried;
    await setSetting('tafsir_view', view);
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
