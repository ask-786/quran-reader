/**
 * Recitation playback.
 *
 * Audio is secondary here. The reader is the app, and recitation exists for the
 * moment someone is unsure how a word is said — so the default gesture is
 * **one verse, then silence**, reached from the tafsir card that is already
 * open on that verse. Playing a whole range is a separate, deliberate act
 * (`playRange`), not what a stray click gives you.
 *
 * # Why blobs and not files
 *
 * `convertFileSrc` yields `asset://localhost/…` on Linux, and WebKitGTK's media
 * pipeline cannot load from a custom URI scheme: `fetch` and `<img>` work,
 * `<audio>` fails with "The operation is not supported". So the bytes come over
 * the IPC bridge and are played from a blob, which the page decodes itself. At
 * 50–420 KB a verse, the copy is not worth avoiding.
 *
 * Two elements, ping-ponged, so a range plays without a gap at each verse: while
 * one sounds, the other is loaded with the next.
 */

import { SvelteMap } from 'svelte/reactivity';
import {
  ensureAyahAudio,
  getAyahsForSurah,
  listReciters,
  prefetchAyahAudio,
  readAyahAudio,
} from '$lib/api/db';
import { settingsStore } from './settings.svelte';
import { surahsStore } from './surahs.svelte';
import type { Ayah, Reciter } from '$lib/types/database';

/**
 * How far ahead of the playhead to pull verses into the cache, in range mode.
 *
 * Five verses is roughly a minute of recitation of buffer — enough that a slow
 * moment on the network is absorbed between verses rather than heard as a gap,
 * and still nowhere near downloading the Surah behind the reader's back.
 *
 * Fetched **one at a time**, and abandoned the moment the playhead moves. Five
 * parallel requests at every verse boundary is a burst against someone else's
 * CDN for files the app does not host, and it would compete with the verse
 * actually being waited on.
 */
const PREFETCH_AHEAD = 5;

/**
 * `single` plays the verse and stops — the pronunciation check. `range` walks
 * the queue, repeats, prefetches and asks the reader to be followed. Almost
 * everything that distinguishes a player from a clarification hangs off this.
 */
export type PlaybackMode = 'single' | 'range';

class PlaybackStore {
  /** The catalogue, loaded once. Empty until `init`. */
  reciters = $state<Reciter[]>([]);

  /** Ayah ids of whatever range is queued, in reading order. Only `range` mode
   *  reads it. */
  queue = $state<number[]>([]);

  /** Surah and verse number for each queued id, so the listen panel can say
   *  what is playing without the caller having to hand it the Ayah rows. */
  #where = new SvelteMap<number, { surahId: number; ayahNumber: number }>();

  /** The verse being recited, or null when nothing is loaded. */
  currentAyahId = $state<number | null>(null);

  mode = $state<PlaybackMode>('single');
  playing = $state(false);
  /** Waiting on a fetch. Distinct from `playing` so a caller can say "getting
   *  the verse" rather than show a stalled play button. */
  loading = $state(false);

  /**
   * The verse is not cached and downloads are turned off. Not an error: the UI
   * offers to turn them on, naming the host, which is the one moment the reader
   * gets to decide whether this app talks to the network at all.
   */
  needsPermission = $state(false);

  error = $state<string | null>(null);

  /**
   * Position and length of the verse being recited, in seconds.
   *
   * Exposed because a verse is not a uniform thing: al-Fatiha 1 is five
   * seconds and 2:282 runs for minutes, and someone checking one phrase in the
   * longest āya in the Quran should not have to sit through the rest of it.
   */
  currentTime = $state(0);
  duration = $state(0);

  /** Repeats still owed on the current verse, under `repeat: ayah`. */
  #repeatsLeft = 0;

  #a: HTMLAudioElement | null = null;
  #b: HTMLAudioElement | null = null;
  /** Which of the two is the one currently sounding. */
  #activeIsA = true;
  /** Object URLs handed to each element, so the previous one is revoked when
   *  its element is pointed at a new verse. Leaking these leaks the audio. */
  #urls = new Map<HTMLAudioElement, string>();
  /** The verse `#idle` has been loaded with, so a swap knows whether it can use
   *  what is already buffered. */
  #preloadedAyahId: number | null = null;
  #pauseTimer: ReturnType<typeof setTimeout> | null = null;
  /** Bumped whenever a new prefetch run starts, so the one in flight can see
   *  that the playhead has moved and give up rather than fetching verses the
   *  reader has already gone past. */
  #prefetchRun = 0;

  async init() {
    if (this.reciters.length) return;
    try {
      this.reciters = await listReciters();
    } catch (err) {
      console.error('Failed to load reciters', err);
    }
  }

  get reciter(): Reciter | null {
    const id = settingsStore.current.reciter_id;
    if (id === null) return null;
    return this.reciters.find((r) => r.id === id) ?? null;
  }

  /** Recitation exists in the UI only once a reciter has been chosen. */
  get enabled(): boolean {
    return this.reciter !== null;
  }

  isPlaying(ayahId: number): boolean {
    return this.playing && this.currentAyahId === ayahId;
  }

  isCurrent(ayahId: number): boolean {
    return this.currentAyahId === ayahId;
  }

  /** Whether the reader should be scrolled along. Only a range walks away from
   *  where the reader is looking; a single verse is already on screen. */
  get shouldFollow(): boolean {
    return this.mode === 'range' && settingsStore.current.audio_follow;
  }

  setQueue(ayahs: Ayah[]) {
    const ids = ayahs.map((a) => a.id);
    this.queue = ids;
    this.#where.clear();
    for (const a of ayahs) {
      this.#where.set(a.id, { surahId: a.surah_id, ayahNumber: a.ayah_number });
    }
    if (this.mode === 'range' && this.currentAyahId !== null && !ids.includes(this.currentAyahId)) {
      this.stop();
    }
  }

  /**
   * Listen to a whole Surah, whichever range happens to be open.
   *
   * Fetched rather than sliced out of the open range on purpose: the header
   * this is reached from can be a Surah that only *starts* inside a Juz, and
   * "listen to Al-Kahf" should mean Al-Kahf and not the part of it that shares
   * a page with something else.
   */
  async playSurah(surahId: number) {
    if (!this.enabled) return;
    try {
      const ayahs = await getAyahsForSurah(surahId);
      if (!ayahs.length) return;
      this.setQueue(ayahs);
      await this.playRange(ayahs[0].id);
    } catch (err) {
      this.#fail(err);
    }
  }

  /** "Al-Baqara 255" for whatever is loaded, or null when nothing is. */
  get label(): string | null {
    if (this.currentAyahId === null) return null;
    const where = this.#where.get(this.currentAyahId);
    if (!where) return null;
    const surah = surahsStore.get(where.surahId);
    return `${surah?.transliteration ?? where.surahId} ${where.ayahNumber}`;
  }

  /** The Surah the queue is in, for the panel's heading. */
  get queueLabel(): string | null {
    const first = this.queue[0];
    if (first === undefined) return null;
    const where = this.#where.get(first);
    if (!where) return null;
    return surahsStore.get(where.surahId)?.transliteration ?? null;
  }

  // ===========================================================================
  // TRANSPORT
  // ===========================================================================

  /**
   * Play one verse and stop.
   *
   * The pronunciation check: you were reading, a word did not sit right, you
   * hear that verse and carry on reading. Repeat settings still apply — hearing
   * the same āya three times is the same question asked three times — but
   * nothing runs on into the next verse.
   */
  async playOne(ayahId: number) {
    if (!this.enabled) return;
    this.#clearPause();
    this.mode = 'single';
    this.#repeatsLeft = this.#repeatsFor();
    await this.#load(ayahId, true);
  }

  /** Play from this verse through the rest of the queue. */
  async playRange(ayahId: number) {
    if (!this.enabled) return;
    this.#clearPause();
    this.mode = 'range';
    this.#repeatsLeft = this.#repeatsFor();
    await this.#load(ayahId, true);
  }

  /** Play/pause for whichever verse is loaded. */
  async toggle(ayahId?: number) {
    if (!this.enabled) return;
    if (ayahId !== undefined && this.currentAyahId !== ayahId) {
      await this.playOne(ayahId);
      return;
    }
    if (this.playing) {
      this.pause();
      return;
    }
    if (this.currentAyahId === null) {
      const first = this.queue[0];
      if (first !== undefined) await this.playRange(first);
      return;
    }
    await this.resume();
  }

  pause() {
    this.#clearPause();
    this.#active?.pause();
    this.playing = false;
  }

  async resume() {
    if (this.currentAyahId === null) return;
    const el = this.#active;
    // No source yet means the last attempt never got a file — a fetch that
    // failed, or downloads turned off. Go through `#load` again rather than
    // pressing play on an empty element.
    if (!el || !el.src) {
      await this.#load(this.currentAyahId, true);
      return;
    }
    this.#apply(el);
    try {
      await el.play();
      this.playing = true;
    } catch (err) {
      this.#fail(err);
    }
  }

  stop() {
    this.#clearPause();
    if (this.#a) this.#a.pause();
    if (this.#b) this.#b.pause();
    this.playing = false;
    this.loading = false;
    this.currentAyahId = null;
    this.needsPermission = false;
    this.error = null;
    this.currentTime = 0;
    this.duration = 0;
    this.#repeatsLeft = 0;
    this.#preloadedAyahId = null;
  }

  /**
   * Stop, but only playback a tafsir card started.
   *
   * Called when the last tafsir surface closes. A range playing from the listen
   * panel has its own surface and its own stop, and closing a commentary card
   * is not a reason to silence it.
   */
  stopIfSingle() {
    if (this.mode === 'single') this.stop();
  }

  /** Next verse in the queue, ignoring repeats still owed — this is the reader
   *  saying "move on", not the player finishing. */
  async next() {
    const id = this.#neighbour(1);
    if (id === null) return;
    await this.playRange(id);
  }

  async previous() {
    const id = this.#neighbour(-1);
    if (id === null) return;
    await this.playRange(id);
  }

  /**
   * Jump to a point in the verse being recited.
   *
   * The reason a bar exists at all rather than just play/pause: the longest
   * Ayah in the Quran (2:282) is minutes of recitation, and "how is this word
   * said" is a question about one phrase in it.
   */
  seek(seconds: number) {
    const el = this.#active;
    if (!el || !Number.isFinite(this.duration) || this.duration <= 0) return;
    const clamped = Math.min(Math.max(seconds, 0), this.duration);
    el.currentTime = clamped;
    this.currentTime = clamped;
  }

  /** Turn downloads on and pick up where the missing verse stopped playback. */
  async allowDownloadsAndRetry() {
    await settingsStore.setAudioDownloadsAllowed(true);
    this.needsPermission = false;
    if (this.currentAyahId !== null) await this.#load(this.currentAyahId, true);
  }

  // ===========================================================================
  // INTERNALS
  // ===========================================================================

  get #active(): HTMLAudioElement | null {
    return this.#activeIsA ? this.#a : this.#b;
  }

  get #idle(): HTMLAudioElement | null {
    return this.#activeIsA ? this.#b : this.#a;
  }

  /** Create the pair on first use. Not at construction: this module is
   *  evaluated where there is no `Audio`. */
  #ensureElements() {
    if (typeof window === 'undefined') return;
    if (!this.#a) this.#a = this.#makeElement();
    if (!this.#b) this.#b = this.#makeElement();
  }

  #makeElement(): HTMLAudioElement {
    const el = new Audio();
    el.preload = 'auto';
    el.addEventListener('ended', () => void this.#onEnded(el));
    // Guarded on the active element throughout: the idle one is buffering the
    // next verse and would otherwise report its position as this one's.
    el.addEventListener('timeupdate', () => {
      if (el === this.#active) this.currentTime = el.currentTime;
    });
    const readDuration = () => {
      // A blob-backed MP3 can report Infinity until it is fully buffered, and
      // a scrubber with an infinite track is worse than one with no track.
      if (el === this.#active && Number.isFinite(el.duration)) this.duration = el.duration;
    };
    el.addEventListener('loadedmetadata', readDuration);
    el.addEventListener('durationchange', readDuration);
    el.addEventListener('error', () => {
      // Only the element actually sounding gets to raise an error. The idle one
      // failing means a preload missed, which the next `#load` will retry.
      if (el === this.#active) this.#fail(el.error?.message ?? 'Playback failed');
    });
    return el;
  }

  /** Point an element at these bytes, releasing whatever it held before. */
  #attach(el: HTMLAudioElement, bytes: ArrayBuffer) {
    const previous = this.#urls.get(el);
    if (previous) URL.revokeObjectURL(previous);
    const url = URL.createObjectURL(new Blob([bytes], { type: 'audio/mpeg' }));
    this.#urls.set(el, url);
    el.src = url;
  }

  #apply(el: HTMLAudioElement) {
    el.volume = settingsStore.current.audio_volume;
    el.playbackRate = settingsStore.current.audio_playback_rate;
  }

  #repeatsFor(): number {
    return settingsStore.current.audio_repeat_mode === 'ayah'
      ? settingsStore.current.audio_repeat_count
      : 1;
  }

  /**
   * Point an element at a verse and play it.
   *
   * Uses the idle element when it already holds this verse, which is the
   * gapless path; otherwise loads the active one, which costs the fetch time an
   * unprefetched verse was always going to cost.
   */
  async #load(ayahId: number, autoplay: boolean) {
    const reciter = this.reciter;
    if (!reciter) return;

    this.#ensureElements();
    this.currentAyahId = ayahId;
    this.error = null;
    this.needsPermission = false;
    // Cleared here rather than on `ended`, so the bar shows the verse that is
    // arriving rather than the tail of the one before it.
    this.currentTime = 0;
    this.duration = 0;

    if (this.#preloadedAyahId === ayahId && this.#idle?.src) {
      this.#activeIsA = !this.#activeIsA;
      this.#preloadedAyahId = null;
      const el = this.#active;
      if (el) {
        this.#apply(el);
        el.currentTime = 0;
        if (autoplay && !(await this.#start(el))) return;
        void this.#prefetchAhead(ayahId);
        return;
      }
    }

    const bitrate = settingsStore.current.audio_bitrate;
    this.loading = true;
    let bytes: ArrayBuffer;
    try {
      // Asked separately from the read so that "not here, and not allowed to
      // fetch it" is an answer rather than an exception.
      const available = await ensureAyahAudio(reciter.slug, bitrate, ayahId);
      if (!available) {
        this.loading = false;
        this.needsPermission = true;
        this.playing = false;
        return;
      }
      bytes = await readAyahAudio(reciter.slug, bitrate, ayahId);
    } catch (err) {
      this.loading = false;
      this.#fail(err);
      return;
    }
    this.loading = false;

    const el = this.#active;
    if (!el) return;
    this.#attach(el, bytes);
    this.#apply(el);
    if (autoplay && !(await this.#start(el))) return;
    void this.#prefetchAhead(ayahId);
  }

  /** Start an element, reporting a refusal once. Returns whether it plays. */
  async #start(el: HTMLAudioElement): Promise<boolean> {
    try {
      await el.play();
      this.playing = true;
      return true;
    } catch (err) {
      this.#fail(err);
      return false;
    }
  }

  /**
   * Load the next verse into the idle element and ask the backend to cache the
   * few after that. Range mode only — a single verse has no next.
   *
   * Not awaited by the caller: these exist so the reader never waits, and making
   * the current verse wait on them would invert that.
   */
  async #prefetchAhead(fromAyahId: number) {
    const reciter = this.reciter;
    if (!reciter || this.mode !== 'range') return;
    if (!settingsStore.current.audio_downloads_allowed) return;

    const bitrate = settingsStore.current.audio_bitrate;
    const index = this.queue.indexOf(fromAyahId);
    if (index === -1) return;

    const run = ++this.#prefetchRun;
    const nextId = this.queue[index + 1];
    if (nextId === undefined) return;

    try {
      const bytes = await readAyahAudio(reciter.slug, bitrate, nextId);
      // Only if this is still the current run: a verse skipped past while this
      // was in flight would otherwise be loaded into the element that is about
      // to play something else.
      if (run === this.#prefetchRun && this.#idle) {
        this.#attach(this.#idle, bytes);
        this.#preloadedAyahId = nextId;
      }
    } catch {
      // A preload that fails is a verse the player will ask for again when it
      // gets there, and report properly then.
    }

    // Sequential on purpose — see PREFETCH_AHEAD. Each await also gives the run
    // a chance to notice it has been superseded.
    for (let i = 2; i <= PREFETCH_AHEAD; i++) {
      if (run !== this.#prefetchRun) return;
      const id = this.queue[index + i];
      if (id === undefined) return;
      await prefetchAyahAudio(reciter.slug, bitrate, id);
    }
  }

  /** What happens when a verse finishes: repeat it, move on, or stop. */
  async #onEnded(el: HTMLAudioElement) {
    if (el !== this.#active) return;

    const { audio_repeat_mode: mode, audio_repeat_pause_ms: pause } = settingsStore.current;

    if (mode === 'ayah' && this.#repeatsLeft > 1) {
      this.#repeatsLeft -= 1;
      this.#after(pause, async () => {
        el.currentTime = 0;
        this.#apply(el);
        await this.#start(el);
      });
      return;
    }

    // A single verse is done when it has finished being repeated. Running on
    // into the next one is what "play the range" is for, and nothing else
    // should quietly become that.
    //
    // Cleared rather than merely paused, because the mark on the verse means
    // "this is the one sounding" — leaving it on a verse that has finished
    // makes the reader look at a highlight with nothing behind it.
    if (this.mode === 'single') {
      this.stop();
      return;
    }

    const index = this.queue.indexOf(this.currentAyahId ?? -1);
    const nextId = index === -1 ? undefined : this.queue[index + 1];

    if (nextId === undefined) {
      if (mode === 'range' && this.queue.length) {
        const first = this.queue[0];
        this.#after(pause, () => this.#load(first, true));
        return;
      }
      this.playing = false;
      return;
    }

    this.#repeatsLeft = this.#repeatsFor();
    this.#after(pause, () => this.#load(nextId, true));
  }

  /**
   * Run `fn` after the configured silence, or immediately when there is none.
   *
   * A zero-length `setTimeout` would still defer a frame, and doing that between
   * two verses is the gap the two elements exist to avoid.
   */
  #after(ms: number, fn: () => void | Promise<void>) {
    this.#clearPause();
    if (ms <= 0) {
      void fn();
      return;
    }
    this.playing = false;
    this.#pauseTimer = setTimeout(() => {
      this.#pauseTimer = null;
      void fn();
    }, ms);
  }

  #clearPause() {
    if (this.#pauseTimer !== null) {
      clearTimeout(this.#pauseTimer);
      this.#pauseTimer = null;
    }
  }

  #neighbour(step: 1 | -1): number | null {
    if (this.currentAyahId === null) return this.queue[0] ?? null;
    const index = this.queue.indexOf(this.currentAyahId);
    if (index === -1) return null;
    return this.queue[index + step] ?? null;
  }

  #fail(err: unknown) {
    this.playing = false;
    this.loading = false;
    this.error = err instanceof Error ? err.message : String(err);
    console.error('Playback failed', err);
  }
}

export const playbackStore = new PlaybackStore();
