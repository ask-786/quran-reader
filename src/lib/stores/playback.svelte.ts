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
 *
 * # Why a whole Surah is streamed instead
 *
 * Gapless is not seamless. Each per-verse file is cut out of a longer recording,
 * and the breath the reciter took between two verses is what the cut removes —
 * no amount of preloading puts it back. So the listen panel plays a Surah from
 * one recording, streamed straight from its host by a third element, and works
 * out which verse is sounding from timings fetched with it (`audio::surah`). The
 * element streams with range requests, so al-Baqara's 100-odd MB is never
 * waited on, and nothing about the stream is written to disk.
 *
 * Everything else stays on per-verse files: the verse card's single verse, a
 * reciter with no whole-Surah source, "cached only" mode, and a stream that
 * fails — which drops to the verse it was on and carries on from the cache or
 * the per-verse host. A Surah downloaded for offline listening is therefore
 * still a Surah of per-verse files, played the old way when the network is not
 * there to stream from.
 *
 * # Why playing is asked for permission rather than attempted
 *
 * WebKitGTK plays through GStreamer, whose elements are system packages it does
 * not itself depend on. Where the sink is missing, `play()` does not reject and
 * does not raise `error` — it wedges the WebKitWebProcess, and the reader, not
 * just the player, stops responding. There is no catching that from in here: a
 * `try` never reaches its `catch` and a timeout never fires, because this is the
 * code that stops running. So `backend` is asked first, in Rust, and a verse is
 * never loaded when the answer is no. See `src-tauri/src/audio/backend.rs`.
 */

import { SvelteMap } from 'svelte/reactivity';
import {
  audioBackend,
  ensureAyahAudio,
  getAyahsForSurah,
  listReciters,
  prefetchAyahAudio,
  readAyahAudio,
  surahRecitation,
} from '$lib/api/db';
import { settingsStore } from './settings.svelte';
import { surahsStore } from './surahs.svelte';
import type { AudioBackend, Ayah, Reciter, SurahRecitation } from '$lib/types/database';

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

/**
 * A Surah's recording, ready to stream: where each verse starts and ends in the
 * file, in seconds, indexed both ways.
 */
interface SurahStream {
  /** The reciter it was fetched for. A stream outlives a change of reciter
   *  otherwise, and would go on playing the old one. */
  slug: string;
  url: string;
  ids: number[];
  starts: number[];
  ends: number[];
  index: Map<number, number>;
  /** The file's length in seconds, for a bar that spans the whole Surah. */
  length: number;
}

function toStream(slug: string, rec: SurahRecitation): SurahStream {
  return {
    slug,
    url: rec.audio_url,
    ids: rec.verses.map((v) => v.ayah_id),
    starts: rec.verses.map((v) => v.start_ms / 1000),
    ends: rec.verses.map((v) => v.end_ms / 1000),
    index: new Map(rec.verses.map((v, i) => [v.ayah_id, i])),
    length: rec.duration_ms / 1000,
  };
}

/** The verse sounding at `t`: the last one to have started, or the first when
 *  `t` is before any of them. */
function verseAt(stream: SurahStream, t: number): number {
  let lo = 0;
  let hi = stream.starts.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (stream.starts[mid] <= t) lo = mid;
    else hi = mid - 1;
  }
  return lo;
}

class PlaybackStore {
  /** The catalogue, loaded once. Empty until `init`. */
  reciters = $state<Reciter[]>([]);

  /**
   * What the machine can actually play, or null while unasked.
   *
   * Null is treated as playable throughout: a probe that failed to run is not
   * evidence of a missing sink, and silently disabling recitation over it would
   * turn one broken call into a feature nobody can switch back on.
   */
  backend = $state<AudioBackend | null>(null);

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

  /**
   * Position and length in the Surah's recording, while it streams. Zero
   * length otherwise, which is how the listen panel knows a bar spanning the
   * whole Surah has nothing to span: verse by verse there is no one file.
   */
  surahTime = $state(0);
  surahLength = $state(0);

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

  /** The queued Surah's whole recording, when it can be streamed. Null plays
   *  the queue verse by verse. */
  #stream: SurahStream | null = null;
  /** The element the stream plays through, separate from the ping-pong pair so
   *  neither has to know which kind of source it holds. */
  #streamEl: HTMLAudioElement | null = null;
  /** Set once the current pass through a verse has reached its end and been
   *  dealt with — repeated, paused after, or let run on — so a boundary checked
   *  sixty times a second is acted on once. */
  #boundaryHandled = false;
  #frame: number | null = null;

  async init() {
    if (this.backend === null) {
      try {
        this.backend = await audioBackend();
      } catch (err) {
        console.error('Failed to probe the audio backend', err);
      }
    }
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

  /** Whether pressing play is safe. See the note at the top of this file. */
  get playable(): boolean {
    return this.backend?.playable !== false;
  }

  /**
   * Why recitation cannot play, written for the reader, or null when it can.
   *
   * Names what is missing and how to install it, because the reader is the only
   * one who can fix this and "audio unavailable" would send them looking for a
   * bug that is not in this app.
   */
  get unavailable(): { reason: string; install: string | null } | null {
    if (this.playable) return null;
    const missing = this.backend?.missing ?? [];
    return {
      reason: missing.length
        ? `This system is missing ${list(missing)}.`
        : 'This system has no working audio backend.',
      install: this.backend?.install_command ?? null,
    };
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
    if (this.#stream && !ids.includes(this.#stream.ids[0])) {
      this.#releaseStream();
      this.#stream = null;
    }
  }

  /**
   * Listen to a whole Surah, whichever range happens to be open.
   *
   * Fetched rather than sliced out of the open range on purpose: the header
   * this is reached from can be a Surah that only *starts* inside a Juz, and
   * "listen to Al-Kahf" should mean Al-Kahf and not the part of it that shares
   * a page with something else.
   *
   * Streamed from one recording when the reciter has one; see the note at the
   * top of this file.
   */
  async playSurah(surahId: number) {
    if (!this.enabled) return;
    try {
      const ayahs = await getAyahsForSurah(surahId);
      if (!ayahs.length) return;
      this.setQueue(ayahs);
      await this.#openStream(surahId);
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
    // A stream fetched for another reciter: ask again for this one.
    if (this.#stream && this.#stream.slug !== this.reciter?.slug) {
      const surahId = this.#where.get(ayahId)?.surahId;
      if (surahId !== undefined) await this.#openStream(surahId);
    }
    if (this.#streamFor(ayahId)) {
      await this.#streamFrom(ayahId);
      return;
    }
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
    (this.#streaming ? this.#streamEl : this.#active)?.pause();
    this.playing = false;
  }

  async resume() {
    if (this.currentAyahId === null) return;
    if (!this.#guard()) return;
    if (this.#streaming) {
      const el = this.#streamEl;
      if (!el || !el.src) {
        await this.#streamFrom(this.currentAyahId);
        return;
      }
      this.#apply(el);
      if (await this.#start(el)) this.#watch();
      return;
    }
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
    this.#releaseStream();
    this.playing = false;
    this.loading = false;
    this.currentAyahId = null;
    this.needsPermission = false;
    this.error = null;
    this.currentTime = 0;
    this.duration = 0;
    this.surahTime = 0;
    this.surahLength = 0;
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
    if (!Number.isFinite(this.duration) || this.duration <= 0) return;
    const clamped = Math.min(Math.max(seconds, 0), this.duration);
    // Streaming, the verse bar's position is an offset from where that verse
    // starts in the Surah's file.
    const stream = this.#streaming ? this.#stream : null;
    const i =
      stream && this.currentAyahId !== null ? stream.index.get(this.currentAyahId) : undefined;
    const el = stream ? this.#streamEl : this.#active;
    if (!el) return;
    if (stream && i !== undefined) {
      el.currentTime = stream.starts[i] + clamped;
      this.#boundaryHandled = false;
    } else {
      el.currentTime = clamped;
    }
    this.currentTime = clamped;
  }

  /**
   * Jump to a point in the Surah's recording, for the bar that spans all of
   * it. The verse is whichever one that point falls in, so the reader follows
   * the jump as it would a verse the reciter ran on into.
   */
  seekSurah(seconds: number) {
    const stream = this.#streaming ? this.#stream : null;
    const el = this.#streamEl;
    if (!stream || !el || !el.src || this.surahLength <= 0) return;
    const t = Math.min(Math.max(seconds, 0), this.surahLength);
    // A jump made in the silence between two verses is where the reader wants
    // to be now, not after the silence has run out.
    const waiting = this.#pauseTimer !== null;
    this.#clearPause();
    el.currentTime = t;
    const id = stream.ids[verseAt(stream, t)];
    if (id !== this.currentAyahId) {
      this.currentAyahId = id;
      this.#repeatsLeft = this.#repeatsFor();
    }
    this.#boundaryHandled = false;
    this.#syncStreamClock();
    if (waiting) void this.#start(el).then((ok) => ok && this.#watch());
  }

  /** The verse number at a point in the Surah's recording, for the bar to say
   *  where a drag would land before it is let go. */
  verseNumberAt(seconds: number): number | null {
    const stream = this.#streaming ? this.#stream : null;
    if (!stream) return null;
    return this.#where.get(stream.ids[verseAt(stream, seconds)])?.ayahNumber ?? null;
  }

  /** Turn downloads on and pick up where the missing verse stopped playback. */
  async allowDownloadsAndRetry() {
    await settingsStore.setAudioDownloadsAllowed(true);
    this.needsPermission = false;
    if (this.currentAyahId === null) return;
    if (this.mode === 'range') {
      // Turning downloads on is also what lets the Surah stream.
      const surahId = this.#where.get(this.currentAyahId)?.surahId;
      if (surahId !== undefined && !this.#stream) await this.#openStream(surahId);
      await this.playRange(this.currentAyahId);
      return;
    }
    await this.#load(this.currentAyahId, true);
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
    // Before `#ensureElements`, so an unplayable machine never gets as far as
    // constructing an `Audio` at all.
    if (!this.#guard()) return;

    this.#ensureElements();
    // A verse from the card while the Surah streams, or a stream that failed
    // over to this path: either way the stream is not what should be heard.
    this.#streamEl?.pause();
    this.surahLength = 0;
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
      // A play() overtaken by a pause or a new source is rejected as aborted.
      // Nothing failed: whatever overtook it is what the reader asked for.
      if (err instanceof DOMException && err.name === 'AbortError') return false;
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

  // ===========================================================================
  // STREAMING
  // ===========================================================================

  /** Whether the queue is playing from the Surah's recording right now. */
  get #streaming(): boolean {
    return this.mode === 'range' && this.#stream !== null;
  }

  /** The stream, if it can play this verse for the chosen reciter. */
  #streamFor(ayahId: number): SurahStream | null {
    const stream = this.#stream;
    if (!stream || stream.slug !== this.reciter?.slug || !stream.index.has(ayahId)) return null;
    return stream;
  }

  /**
   * Ask for the Surah's recording, leaving `#stream` null when it cannot be
   * streamed — which is never an error the reader sees, because the per-verse
   * path plays the same Surah.
   *
   * Not attempted in "cached only" mode: streaming is the network by
   * definition, and that switch is the reader's promise that there is none.
   */
  async #openStream(surahId: number) {
    this.#releaseStream();
    this.#stream = null;
    const reciter = this.reciter;
    if (!reciter || !settingsStore.current.audio_downloads_allowed) return;
    if (typeof navigator !== 'undefined' && !navigator.onLine) return;
    this.loading = true;
    try {
      const rec = await surahRecitation(reciter.slug, surahId);
      if (rec && rec.verses.length && reciter.slug === this.reciter?.slug) {
        this.#stream = toStream(reciter.slug, rec);
      }
    } catch (err) {
      console.warn('No whole-Surah recording; playing verse by verse', err);
    } finally {
      this.loading = false;
    }
  }

  #ensureStreamElement(): HTMLAudioElement | null {
    if (typeof window === 'undefined') return null;
    if (this.#streamEl) return this.#streamEl;
    const el = new Audio();
    // Fetched as it plays rather than all at once: al-Baqara is over 100 MB,
    // and someone listening to its first page has not asked for the rest.
    el.preload = 'metadata';
    el.addEventListener('timeupdate', () => {
      // Dropping the source on stop fires one last update at 0:00, which would
      // otherwise put verse 1 back as the one sounding.
      if (this.#streaming && el.src && this.currentAyahId !== null) this.#onStreamTime(true);
    });
    el.addEventListener('ended', () => void this.#onStreamEnded());
    // WebKitGTK's MP3 seeking only jumps straight to parts of the file it has
    // already seen; a jump past them is read up to, which on al-Baqara can
    // take seconds. Without this it reads as a frozen bar.
    el.addEventListener('waiting', () => {
      if (this.#streaming) this.loading = true;
    });
    el.addEventListener('playing', () => {
      if (this.#streaming) this.loading = false;
    });
    el.addEventListener('error', () => {
      if (!this.#streaming || !el.src) return;
      // Fall back rather than fail: the per-verse path is the same Surah, and
      // it plays from the cache when the network is what went away.
      console.warn('Streaming the Surah failed; playing verse by verse', el.error);
      const at = this.currentAyahId;
      this.#releaseStream();
      this.#stream = null;
      this.loading = false;
      if (at !== null) void this.#load(at, true);
    });
    this.#streamEl = el;
    return el;
  }

  /** Start or move the stream to the beginning of a verse. */
  async #streamFrom(ayahId: number) {
    const stream = this.#streamFor(ayahId);
    if (!stream) return;
    if (!this.#guard()) return;
    const el = this.#ensureStreamElement();
    if (!el) return;

    // The pair may be mid-verse from the card, or holding a prefetch that the
    // stream makes pointless; the bump abandons any prefetch run in flight.
    this.#a?.pause();
    this.#b?.pause();
    this.#prefetchRun++;
    this.#preloadedAyahId = null;

    const i = stream.index.get(ayahId) ?? 0;
    this.currentAyahId = ayahId;
    this.error = null;
    this.needsPermission = false;
    this.#boundaryHandled = false;

    if (el.src !== stream.url) {
      el.src = stream.url;
      this.loading = true;
    }
    // From the top of the file for the first verse: the recording opens with
    // the isti'adha and basmala, which belong to the Surah though to no verse.
    const from = i === 0 ? 0 : stream.starts[i];
    await this.#seekStream(el, from);
    if (!this.#stillStreaming(stream, el)) return;
    this.#apply(el);
    this.#syncStreamClock();
    if (await this.#start(el)) this.#watch();
  }

  /**
   * Whether a stream start that awaited a seek is still the one wanted. A
   * failure while it waited has already fallen back to per-verse files, and
   * pressing play on the emptied element would put an error over a fallback
   * that is working.
   */
  #stillStreaming(stream: SurahStream, el: HTMLAudioElement): boolean {
    return this.#stream === stream && this.#streaming && el.src === stream.url;
  }

  /**
   * Seek, waiting for metadata first when the element has none yet: before it
   * knows the file's length, a seek is not reliably honoured.
   */
  #seekStream(el: HTMLAudioElement, t: number): Promise<void> {
    if (el.readyState >= HTMLMediaElement.HAVE_METADATA) {
      el.currentTime = t;
      return Promise.resolve();
    }
    return new Promise((resolve) => {
      const done = () => {
        el.removeEventListener('loadedmetadata', done);
        el.removeEventListener('error', done);
        if (!el.error) el.currentTime = t;
        resolve();
      };
      el.addEventListener('loadedmetadata', done);
      el.addEventListener('error', done);
    });
  }

  /**
   * Check the playhead against the verse it is in. Runs every frame while the
   * stream plays, because `timeupdate` fires only about four times a second and
   * a repeat that waited for it would play a quarter-second of the next verse
   * first. The bar's clock is only touched from `timeupdate`: re-rendering it
   * every frame buys nothing a reader could see.
   */
  #onStreamTime(updateClock: boolean) {
    const stream = this.#stream;
    const el = this.#streamEl;
    if (!stream || !el || this.#pauseTimer !== null || el.seeking) return;
    const t = el.currentTime;
    const i = this.currentAyahId === null ? undefined : stream.index.get(this.currentAyahId);

    if (i !== undefined && !this.#boundaryHandled && t >= stream.ends[i]) {
      this.#boundaryHandled = true;
      const { audio_repeat_mode: mode, audio_repeat_pause_ms: pause } = settingsStore.current;
      const hasNext = i + 1 < stream.ids.length;

      if (mode === 'ayah' && this.#repeatsLeft > 1) {
        this.#repeatsLeft -= 1;
        el.pause();
        this.#after(pause, () => this.#restartStreamAt(i));
        return;
      }
      if (pause > 0 && hasNext) {
        el.pause();
        this.#after(pause, () => {
          this.#repeatsLeft = this.#repeatsFor();
          return this.#restartStreamAt(i + 1);
        });
        return;
      }
    }

    const at = verseAt(stream, t);
    if (at !== i) {
      // Run on into the next verse, or a seek landed in another one.
      this.currentAyahId = stream.ids[at];
      this.#repeatsLeft = this.#repeatsFor();
      this.#boundaryHandled = false;
      updateClock = true;
    }
    if (updateClock) this.#syncStreamClock();
  }

  /** `fromTop` takes in the lead-in before the first verse, for the whole
   *  Surah coming round again rather than its first verse repeating. */
  async #restartStreamAt(i: number, fromTop = false) {
    const stream = this.#stream;
    const el = this.#streamEl;
    if (!stream || !el) return;
    this.currentAyahId = stream.ids[i];
    this.#boundaryHandled = false;
    await this.#seekStream(el, fromTop ? 0 : stream.starts[i]);
    if (!this.#stillStreaming(stream, el)) return;
    this.#apply(el);
    this.#syncStreamClock();
    if (await this.#start(el)) this.#watch();
  }

  /** Both clocks: the verse's slice of the file, and the file as a whole. */
  #syncStreamClock() {
    const stream = this.#stream;
    const el = this.#streamEl;
    const i =
      stream && this.currentAyahId !== null ? stream.index.get(this.currentAyahId) : undefined;
    if (!stream || !el || i === undefined) return;
    const length = stream.ends[i] - stream.starts[i];
    this.duration = length;
    this.currentTime = Math.min(Math.max(el.currentTime - stream.starts[i], 0), length);
    // The timings end where the last verse does; the file runs a little past.
    this.surahLength = Number.isFinite(el.duration)
      ? Math.max(stream.length, el.duration)
      : stream.length;
    this.surahTime = Math.min(el.currentTime, this.surahLength);
  }

  /** The end of the file: the last verse's repeats, the whole Surah again, or
   *  stop. The same three outcomes `#onEnded` gives the per-verse path. */
  async #onStreamEnded() {
    const stream = this.#stream;
    if (!this.#streaming || !stream) return;
    const { audio_repeat_mode: mode, audio_repeat_pause_ms: pause } = settingsStore.current;
    const last = stream.ids.length - 1;

    // The last verse's end can sit at or past the end of the file, so its
    // boundary may never have been seen in `#onStreamTime`.
    if (mode === 'ayah' && this.#repeatsLeft > 1 && !this.#boundaryHandled) {
      this.#repeatsLeft -= 1;
      this.#after(pause, () => this.#restartStreamAt(last));
      return;
    }
    if (mode === 'range') {
      this.#repeatsLeft = this.#repeatsFor();
      this.#after(pause, () => this.#restartStreamAt(0, true));
      return;
    }
    this.playing = false;
  }

  /** Watch the playhead every frame until playback stops. */
  #watch() {
    if (this.#frame !== null || typeof requestAnimationFrame === 'undefined') return;
    const step = () => {
      this.#frame = null;
      if (!this.#streaming || !this.playing) return;
      this.#onStreamTime(false);
      this.#frame = requestAnimationFrame(step);
    };
    this.#frame = requestAnimationFrame(step);
  }

  /**
   * Stop the stream and drop its source. Pausing alone would leave the element
   * buffering a hundred megabytes nobody is listening to.
   */
  #releaseStream() {
    if (this.#frame !== null) {
      cancelAnimationFrame(this.#frame);
      this.#frame = null;
    }
    this.surahTime = 0;
    this.surahLength = 0;
    const el = this.#streamEl;
    if (!el || !el.src) return;
    el.pause();
    el.removeAttribute('src');
    el.load();
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

  /**
   * Refuse to touch an audio element when the machine cannot play one, leaving
   * the reason where the UI already looks for it.
   */
  #guard(): boolean {
    const blocked = this.unavailable;
    if (!blocked) return true;
    this.playing = false;
    this.loading = false;
    this.error = blocked.reason;
    return false;
  }

  #fail(err: unknown) {
    this.playing = false;
    this.loading = false;
    this.error = err instanceof Error ? err.message : String(err);
    console.error('Playback failed', err);
  }
}

/** "a and b", "a, b and c" — the missing pieces read as a sentence. */
function list(items: string[]): string {
  if (items.length <= 1) return items[0] ?? '';
  return `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`;
}

export const playbackStore = new PlaybackStore();
