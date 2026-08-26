import { listen } from '@tauri-apps/api/event';
import {
  getTafsirs,
  getTafsirForAyah,
  listTafsirPacks,
  installTafsirPack,
  removeTafsirPack,
  setSetting,
} from '$lib/api/db';
import type {
  Tafsir,
  TafsirEntry,
  TafsirPack,
  TafsirPackProgress,
  TafsirView,
} from '$lib/types/database';
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

  /**
   * Editions that can be downloaded, whether or not they already have been.
   *
   * Kept beside `editions` rather than merged into it: an installed edition
   * has an id, a direction and text behind it, and one that is still on a
   * server has a size and a licence. The picker shows both lists, and the
   * difference between them is the whole point of showing the second one.
   */
  packs = $state<TafsirPack[]>([]);

  /** Slug of the edition being downloaded, or null. One at a time — two
   *  concurrent 25 MB downloads help nobody. */
  installing = $state<string | null>(null);

  /** Bytes so far and expected, while `installing`. */
  progress = $state<{ received: number; total: number } | null>(null);

  /** Why the last install failed, cleared when another is started. */
  packError = $state<string | null>(null);

  /**
   * Whether the panel is showing the edition list instead of commentary.
   *
   * Panel-only and session-only: the popover has no room for it, and a reader
   * who opens the app wants the tafsir, not the shop.
   */
  managing = $state(false);

  /** Whether the progress subscription is up. It is never torn down — the
   *  store is a singleton that lives as long as the window does — so this only
   *  has to stop a second `init()` from subscribing twice. */
  #progressBound = false;

  #cache = new Map<string, TafsirEntry | null>();
  /** Guards against an earlier request resolving after a later one. */
  #requestToken = 0;

  /** Whether the side panel is open. Unrelated to the popover. */
  get panelOpen() {
    return settingsStore.current.show_tafsir;
  }

  /**
   * Whether a click on a verse opens its commentary — "tafsir mode".
   *
   * Off by default, and the reason is the reader: the popover answers a
   * question, and a reader who is not asking one still clicks, to place a
   * cursor or dismiss something or for no reason at all. Every one of those
   * clicks putting a card over the text makes the reader worse. The explicit
   * routes in — the per-Ayah button, and `t` — are open whatever this says.
   */
  get clickOpens() {
    return settingsStore.current.tafsir_click;
  }

  get view(): TafsirView {
    return settingsStore.current.tafsir_view;
  }

  get width() {
    return settingsStore.current.tafsir_panel_width;
  }

  /**
   * Live width while the resize handle is being dragged; null when the stored
   * width is in force.
   *
   * Session-only, and deliberately here rather than inside TafsirPanel: the
   * panel overlays the reader now, so the reader's right-edge controls offset
   * themselves by the panel's width to stay reachable. They have to follow the
   * drag frame by frame, and a value private to the panel cannot be read by
   * anything outside it.
   */
  dragWidth = $state<number | null>(null);

  /** What the panel is actually this wide right now, drag included. */
  get liveWidth() {
    return this.dragWidth ?? this.width;
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
    await this.refreshPacks();

    // Subscribed once for the life of the app rather than per download: the
    // events carry their own slug, and a listener torn down and rebuilt around
    // each install can miss the first chunks of the next one.
    if (!this.#progressBound) {
      this.#progressBound = true;
      await listen<TafsirPackProgress>('tafsir-pack-progress', ({ payload }) => {
        if (payload.slug !== this.installing) return;
        this.progress = { received: payload.received, total: payload.total };
      });
    }

    this.ready = true;
  }

  async refreshPacks() {
    try {
      this.packs = await listTafsirPacks();
    } catch (err) {
      console.error('Failed to list tafsir packs', err);
      this.packs = [];
    }
  }

  /**
   * Download and install one edition, then make it the active one.
   *
   * Switching to it afterwards is the point of having asked for it — a
   * download that finishes and changes nothing on screen reads as a failure.
   */
  async installPack(slug: string) {
    if (this.installing) return;
    this.installing = slug;
    this.progress = null;
    this.packError = null;
    try {
      const id = await installTafsirPack(slug);
      this.editions = await getTafsirs();
      await this.refreshPacks();
      await this.setEdition(id);
    } catch (err) {
      console.error('Failed to install tafsir pack', err);
      // The backend's message is the useful one here — it distinguishes a
      // failed download from a file that did not match its hash.
      this.packError = String(err);
    } finally {
      this.installing = null;
      this.progress = null;
    }
  }

  /** Remove an installed edition, moving off it first if it was in use. */
  async removePack(slug: string) {
    const removed = this.editions.find((t) => t.slug === slug);
    try {
      await removeTafsirPack(slug);
      this.editions = await getTafsirs();
      await this.refreshPacks();
      if (removed && settingsStore.current.tafsir_id === removed.id) {
        // `active` already falls back to the first edition, but the stored
        // setting would otherwise keep pointing at an id that no longer
        // exists.
        const fallback = this.editions[0];
        if (fallback) await this.setEdition(fallback.id);
      }
      this.#cache.clear();
      this.entry = null;
    } catch (err) {
      console.error('Failed to remove tafsir pack', err);
      this.packError = String(err);
    }
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

  /**
   * The same thing, asked for by clicking the verse itself rather than a
   * control. Silent unless tafsir mode is on — a click on running text is too
   * cheap and too accidental to be a request on its own.
   */
  openFromClick(ayahId: number, anchor: HTMLElement | null = null) {
    if (!this.clickOpens) return;
    this.openFor(ayahId, anchor);
  }

  closePopover() {
    this.selection = null;
  }

  /**
   * What the toolbar button and `t` do: in panel view, toggle the panel; in
   * popover view, turn tafsir mode on or off.
   *
   * One control, one meaning. Turning the mode on also opens on wherever the
   * reader is, so the key still answers "what does this verse say" in one
   * press rather than arming something you then have to click. Dismissing that
   * card — Escape, or its close button — leaves the mode on, because the mode
   * is not the card: it is whether the next click asks for one.
   */
  toggle() {
    if (this.view === 'panel') {
      void this.setPanelOpen(!this.panelOpen);
      return;
    }
    const on = !this.clickOpens;
    void this.setClickOpens(on);
    if (on) this.openAtReaderPosition();
  }

  /** Open on whatever the reader is looking at — the only defensible target
   *  when nothing was clicked. */
  openAtReaderPosition() {
    const ayahId = readerPosition.ayahId;
    if (ayahId === null) return;
    // Anchor to the Ayah's own element when the reader has one rendered; the
    // popover falls back to a viewport-centred position when it does not.
    const anchor = document.querySelector<HTMLElement>(`[data-ayah-id="${ayahId}"]`);
    this.selection = { ayahId, anchor };
  }

  /** Turning the mode off takes the open card with it: leaving one behind
   *  would say the mode was still on. */
  async setClickOpens(on: boolean) {
    settingsStore.current.tafsir_click = on;
    if (!on) this.closePopover();
    await setSetting('tafsir_click', String(on));
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
