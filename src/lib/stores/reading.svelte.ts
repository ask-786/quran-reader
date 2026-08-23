import { clearReadingHistory, getReadingHistory, recordReadingPosition } from '$lib/api/db';
import type { ReadingScope, ReadingSession } from '$lib/types/database';

/**
 * Where the reader is, written through to the database, and the history of
 * sittings that falls out of it.
 *
 * The reader publishes a centred Ayah on almost every scroll frame, and every
 * one of those is a candidate position. Two delays turn that stream into
 * something worth storing:
 *
 * - `MOVE_DELAY` — while reading inside a range that has already been recorded,
 *   a position is written once the reader settles on an Ayah. Short, because
 *   the cost of losing it is resuming a screen off.
 *
 * - `ENTER_DELAY` — the *first* write in a range waits longer. Paging through
 *   the sidebar puts you in and out of half a dozen Surahs in a few seconds,
 *   and none of them are places you were reading; making the first write earn
 *   a few seconds of attention is what keeps those out of the history.
 *
 * A range that never earned its first write is also the one case where a
 * pending position is *dropped* rather than flushed on the way out — see
 * `flush()`.
 */
const ENTER_DELAY = 3000;
const MOVE_DELAY = 900;

/** How many sittings the Recent list asks for. */
const HISTORY_LIMIT = 40;

type Pending = {
  scope: ReadingScope;
  scopeId: number;
  ayahId: number;
};

const keyOf = (scope: ReadingScope, scopeId: number) => `${scope}:${scopeId}`;

class ReadingStore {
  history = $state<ReadingSession[]>([]);
  historyLoading = $state(false);

  #pending: Pending | null = null;
  #timer: ReturnType<typeof setTimeout> | undefined;

  /** The range keys already written to in this app run — see `ENTER_DELAY`. */
  #recorded = new Set<string>();
  /** The last position actually written, so a still reader doesn't re-write it. */
  #lastWritten: string | null = null;

  /** Reload on the next `refreshHistory()`; set by every successful write. */
  #historyStale = true;

  /**
   * Note the Ayah the reader is looking at. Cheap and safe to call on every
   * scroll — it only schedules, and identical positions are dropped outright.
   */
  note(scope: ReadingScope, scopeId: number, ayahId: number) {
    const key = keyOf(scope, scopeId);
    if (this.#lastWritten === `${key}:${ayahId}`) return;

    this.#pending = { scope, scopeId, ayahId };
    clearTimeout(this.#timer);
    this.#timer = setTimeout(
      () => void this.#write(),
      this.#recorded.has(key) ? MOVE_DELAY : ENTER_DELAY,
    );
  }

  /**
   * Write a pending position now instead of waiting out its delay — called when
   * the reader leaves a range or the window goes away, both of which would
   * otherwise lose the last few seconds of reading.
   *
   * A range still waiting on its `ENTER_DELAY` is dropped rather than written:
   * it was passed through, not read, and that is exactly the case the longer
   * first delay exists to filter out.
   */
  flush() {
    clearTimeout(this.#timer);
    const pending = this.#pending;
    if (!pending) return;
    if (!this.#recorded.has(keyOf(pending.scope, pending.scopeId))) {
      this.#pending = null;
      return;
    }
    void this.#write();
  }

  /** Drop anything scheduled without writing it. */
  cancel() {
    clearTimeout(this.#timer);
    this.#pending = null;
  }

  async #write() {
    const pending = this.#pending;
    if (!pending) return;
    this.#pending = null;

    const key = keyOf(pending.scope, pending.scopeId);
    try {
      await recordReadingPosition(pending.scope, pending.scopeId, pending.ayahId);
      this.#recorded.add(key);
      this.#lastWritten = `${key}:${pending.ayahId}`;
      this.#historyStale = true;
    } catch (err) {
      console.error('Failed to record reading position', err);
    }
  }

  /**
   * Load the Recent list if anything has been written since it was last read.
   * Deliberately pull-based: the list only has to be right when it is on
   * screen, and reloading it on every scroll would re-render the sidebar
   * throughout a reading session for no one's benefit.
   */
  async refreshHistory(force = false) {
    if (!force && !this.#historyStale && this.history.length > 0) return;
    this.historyLoading = true;
    try {
      this.history = await getReadingHistory(HISTORY_LIMIT);
      this.#historyStale = false;
    } catch (err) {
      console.error('Failed to load reading history', err);
    } finally {
      this.historyLoading = false;
    }
  }

  async clearHistory() {
    try {
      await clearReadingHistory();
      this.history = [];
      this.#historyStale = false;
    } catch (err) {
      console.error('Failed to clear reading history', err);
    }
  }
}

export const readingStore = new ReadingStore();
