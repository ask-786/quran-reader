/**
 * The navigator: mode tabs, a filter, a keyboard cursor, and the `2:255` /
 * `p255` parsing — all of it in one place because two surfaces run on it.
 *
 * The sidebar is the docked, mouse-first column; the command palette is the
 * keyboard overlay. They render differently and they answer to different keys,
 * but "what is in this list, what is the cursor on, and where does Enter go"
 * is the same question in both, and it is answered here rather than twice.
 *
 * Each surface owns its own instance: switching to the Juz tab in the palette
 * has no business rearranging the sidebar behind it.
 */

import { resolve } from '$app/paths';
import { readingStore } from '$lib/stores/reading.svelte';
import { surahsStore } from '$lib/stores/surahs.svelte';
import type { Surah } from '$lib/types/database';
import { stripTashkeel } from '$lib/utils/arabic-text';
import { rangeLabel, relativeTime, scopeHref, scopeLabel } from '$lib/utils/reading-scope';

export type NavMode = 'surah' | 'juz' | 'hizb' | 'recent';

export const NAV_MODES: { id: NavMode; label: string }[] = [
  { id: 'surah', label: 'Surah' },
  { id: 'juz', label: 'Juz' },
  { id: 'hizb', label: 'Hizb' },
  { id: 'recent', label: 'Recent' },
];

export const MAX_PAGE = 604;
const JUZ_LIST = Array.from({ length: 30 }, (_, i) => i + 1);
const HIZB_LIST = Array.from({ length: 60 }, (_, i) => i + 1);

/**
 * One row of the result list. The jump row is the parsed `2:255` / `p255`
 * target and only ever appears first — everything else is a list item.
 *
 * `active` marks the range that is currently open in the reader; it is not the
 * cursor. Where you are and what you are about to pick are different things,
 * and the list shows them differently.
 */
export type NavEntry =
  | { kind: 'jump'; key: string; href: string; label: string; detail: string }
  | { kind: 'surah'; key: string; href: string; surah: Surah; active: boolean }
  | {
      kind: 'unit';
      key: string;
      href: string;
      unit: 'juz' | 'hizb';
      n: number;
      active: boolean;
    }
  | {
      kind: 'recent';
      key: string;
      href: string;
      scope: string;
      when: string;
      range: string;
      page: number | null;
    };

/** What the filter box parses to, if it parses to anything at all. */
type Parsed =
  { type: 'ayah'; surahId: number; ayahNumber: number } | { type: 'page'; page: number } | null;

/** Unique per instance, so two panels on screen don't share option element ids. */
let nextPanelId = 1;

export class Navigator {
  readonly panelId = `nav${nextPanelId++}`;

  /**
   * Whether a row is always under the cursor. True for the palette, where Enter
   * is the whole point and landing on a dead key is a bug; false for the
   * sidebar, where the list is something you look at and a bare Enter should
   * not fire off a Surah you never chose.
   */
  readonly autoSelect: boolean;

  mode = $state<NavMode>('surah');
  query = $state('');

  /**
   * The cursor as *intent*, which is why it is allowed to be stale or out of
   * range: `activeIndex` below clamps it against the list that actually exists
   * right now, so a list that shrinks under the cursor can't strand it.
   */
  cursor = $state(-1);

  /** Kept current by the host component; see `setRoute`. */
  routeId = $state<string | null>(null);
  routeParam = $state(0);

  constructor(options: { autoSelect?: boolean } = {}) {
    this.autoSelect = options.autoSelect ?? false;
  }

  setRoute(routeId: string | null, param: number) {
    if (this.routeId !== routeId) this.routeId = routeId;
    if (this.routeParam !== param) this.routeParam = param;
  }

  // ---------------------------------------------------------------- filtering

  #trimmed = $derived(this.query.trim());

  #surahs = $derived.by(() => {
    const q = this.#trimmed.toLowerCase();
    if (!q) return surahsStore.list;
    // Both sides unvocalised: nobody types the Mushaf's harakat into a filter box.
    const qArabic = stripTashkeel(this.#trimmed);
    return surahsStore.list.filter(
      (s) =>
        stripTashkeel(s.name_ar).includes(qArabic) ||
        s.transliteration.toLowerCase().includes(q) ||
        s.name_en.toLowerCase().includes(q) ||
        String(s.id) === q,
    );
  });

  #units = $derived.by(() => {
    const list = this.mode === 'hizb' ? HIZB_LIST : JUZ_LIST;
    const q = this.#trimmed;
    return q ? list.filter((n) => String(n).includes(q)) : list;
  });

  #recent = $derived.by(() => {
    const q = this.#trimmed.toLowerCase();
    const rows = readingStore.history.map((session) => ({
      session,
      scope: scopeLabel(session.scope, session.scope_id, surahsStore),
      range: rangeLabel(session.start, session.end),
    }));
    if (!q) return rows;
    return rows.filter((r) => r.scope.toLowerCase().includes(q) || r.range.includes(q));
  });

  // ------------------------------------------------------------------- go-to

  /** The Surah the reader has open, or 0 — the only context a bare number has. */
  #activeSurahId = $derived(this.routeId === '/surah/[id]' ? this.routeParam : 0);

  /** Whether a bare number can mean anything here. Drives the hint line too. */
  get inSurah() {
    return this.#activeSurahId > 0;
  }

  #parsed = $derived.by((): Parsed => {
    const input = this.#trimmed;

    const pageMatch = input.match(/^p\s*(\d{1,3})$/i);
    if (pageMatch) return { type: 'page', page: Number(pageMatch[1]) };

    const pair = input.match(/^(\d{1,3})\s*[:./]\s*(\d{1,3})$/);
    if (pair) return { type: 'ayah', surahId: Number(pair[1]), ayahNumber: Number(pair[2]) };

    if (/^\d{1,3}$/.test(input)) {
      // A bare number only means "ayah in this surah" — in Juz/Hizb/page mode
      // there is no single Surah to resolve it against, so it stays a filter.
      if (!this.#activeSurahId) return null;
      return { type: 'ayah', surahId: this.#activeSurahId, ayahNumber: Number(input) };
    }

    return null;
  });

  /**
   * The parsed target as either a row to offer or a reason it can't be one.
   * Out-of-range is worth saying out loud ("Al-Baqarah has 286 verses"); text
   * that was never a jump in the first place just isn't one, silently.
   */
  #jump = $derived.by((): { entry: NavEntry } | { error: string } | null => {
    const target = this.#parsed;
    if (!target) return null;

    if (target.type === 'page') {
      if (target.page < 1 || target.page > MAX_PAGE) {
        return { error: `Page must be between 1 and ${MAX_PAGE}` };
      }
      return {
        entry: {
          kind: 'jump',
          key: `jump:page:${target.page}`,
          href: resolve('/page/[id]', { id: String(target.page) }),
          label: `Page ${target.page}`,
          detail: 'Mushaf page',
        },
      };
    }

    const surah = surahsStore.get(target.surahId);
    if (!surah) return { error: `Surah ${target.surahId} doesn't exist` };
    if (target.ayahNumber < 1 || target.ayahNumber > surah.verses_count) {
      return { error: `${surah.transliteration} has ${surah.verses_count} verses` };
    }
    return {
      entry: {
        kind: 'jump',
        key: `jump:ayah:${surah.id}:${target.ayahNumber}`,
        href: resolve(`/surah/[id]?ayah=${target.ayahNumber}`, { id: String(surah.id) }),
        label: `${surah.transliteration} ${surah.id}:${target.ayahNumber}`,
        detail: 'Ayah',
      },
    };
  });

  /** The message under the filter box: an error if there is one, else guidance. */
  get error() {
    const jump = this.#jump;
    return jump && 'error' in jump ? jump.error : '';
  }

  // ----------------------------------------------------------------- entries

  #activeJuz = $derived(this.routeId === '/juz/[id]' ? this.routeParam : 0);
  #activeHizb = $derived(this.routeId === '/hizb/[id]' ? this.routeParam : 0);

  #rows = $derived.by((): NavEntry[] => {
    switch (this.mode) {
      case 'surah':
        return this.#surahs.map((surah) => ({
          kind: 'surah' as const,
          key: `surah:${surah.id}`,
          href: resolve('/surah/[id]', { id: String(surah.id) }),
          surah,
          active: surah.id === this.#activeSurahId,
        }));
      case 'juz':
        return this.#units.map((n) => ({
          kind: 'unit' as const,
          key: `juz:${n}`,
          href: resolve('/juz/[id]', { id: String(n) }),
          unit: 'juz' as const,
          n,
          active: n === this.#activeJuz,
        }));
      case 'hizb':
        return this.#units.map((n) => ({
          kind: 'unit' as const,
          key: `hizb:${n}`,
          href: resolve('/hizb/[id]', { id: String(n) }),
          unit: 'hizb' as const,
          n,
          active: n === this.#activeHizb,
        }));
      case 'recent':
        return this.#recent.map(({ session, scope, range }) => ({
          kind: 'recent' as const,
          key: `recent:${session.id}`,
          // Straight to the Ayah this sitting reached, rather than to the
          // range's current position: an older entry has to be able to take you
          // back to where *it* left off.
          href: scopeHref(session.scope, session.scope_id, session.end.id),
          scope,
          when: relativeTime(session.updated_at),
          range,
          // Where it ended up in the Mushaf, which is how you find it in a
          // printed copy. Omitted on a page-scoped sitting, whose heading is
          // already that page number.
          page: session.scope === 'page' ? null : session.end.page,
        }));
    }
  });

  entries = $derived.by((): NavEntry[] => {
    const jump = this.#jump;
    return jump && 'entry' in jump ? [jump.entry, ...this.#rows] : this.#rows;
  });

  /** True while the list is waiting on something rather than genuinely empty. */
  loading = $derived(
    this.mode === 'surah'
      ? surahsStore.loading
      : this.mode === 'recent'
        ? readingStore.historyLoading && readingStore.history.length === 0
        : false,
  );

  loadError = $derived(this.mode === 'surah' ? surahsStore.error : null);

  // ------------------------------------------------------------------ cursor

  /**
   * The cursor clamped to the list that exists right now. `cursor` is intent
   * and may point anywhere; this is the row actually marked, and it is the only
   * one the UI and Enter should ever look at.
   */
  activeIndex = $derived.by(() => {
    const n = this.entries.length;
    if (n === 0) return -1;
    if (this.cursor < 0) return this.autoSelect ? 0 : -1;
    return Math.min(this.cursor, n - 1);
  });

  optionId(i: number) {
    return `${this.panelId}-option-${i}`;
  }

  get activeOptionId() {
    return this.activeIndex >= 0 ? this.optionId(this.activeIndex) : undefined;
  }

  /** The row Enter should open: the cursor's, or the jump row when idle. */
  get target(): NavEntry | undefined {
    if (this.activeIndex >= 0) return this.entries[this.activeIndex];
    const first = this.entries[0];
    return first?.kind === 'jump' ? first : undefined;
  }

  get targetIndex() {
    if (this.activeIndex >= 0) return this.activeIndex;
    return this.entries[0]?.kind === 'jump' ? 0 : -1;
  }

  move(delta: number) {
    const n = this.entries.length;
    if (n === 0) return;
    const from = this.activeIndex;
    if (from < 0) {
      // Entering the list from the box: down lands on the first row, up on the last.
      this.cursor = delta > 0 ? 0 : n - 1;
    } else {
      // Clamped, not wrapped: with 114 Surahs, falling off the top and landing
      // at the bottom loses your place more than it helps.
      this.cursor = Math.min(Math.max(from + delta, 0), n - 1);
    }
  }

  moveTo(index: number) {
    this.cursor = index;
  }

  /** Retire the cursor — used when the list is about to be a different list. */
  resetCursor() {
    this.cursor = -1;
  }

  setQuery(value: string) {
    if (this.query === value) return;
    this.query = value;
    // Re-typing means you are choosing again, not yet navigating: the row the
    // cursor pointed at has moved or gone.
    this.resetCursor();
  }

  setMode(mode: NavMode) {
    if (this.mode === mode) return;
    this.mode = mode;
    this.query = '';
    this.resetCursor();
    if (mode === 'recent') void readingStore.refreshHistory();
  }

  cycleMode(delta: number) {
    const i = NAV_MODES.findIndex((m) => m.id === this.mode);
    // Wrapped, unlike the cursor: there are four tabs and you can see all of
    // them, so Tab past the end obviously comes back round.
    const next = (i + delta + NAV_MODES.length) % NAV_MODES.length;
    this.setMode(NAV_MODES[next].id);
  }

  /**
   * Open on the tab the reader is already in, with the cursor on the range they
   * are already reading — so the palette opens *where you are* and the next
   * Surah is one keystroke away. Page routes have no tab of their own and fall
   * back to Surah.
   *
   * The cursor is set by arithmetic rather than by searching `entries`, because
   * an unfiltered list is 1..N in order and this has to be right even when it
   * is called before the Surah list has finished loading.
   */
  syncToRoute() {
    this.query = '';
    switch (this.routeId) {
      case '/juz/[id]':
        this.mode = 'juz';
        this.cursor = this.routeParam - 1;
        return;
      case '/hizb/[id]':
        this.mode = 'hizb';
        this.cursor = this.routeParam - 1;
        return;
      case '/surah/[id]':
        this.mode = 'surah';
        this.cursor = this.routeParam - 1;
        return;
      default:
        this.mode = 'surah';
        this.cursor = -1;
    }
  }
}
