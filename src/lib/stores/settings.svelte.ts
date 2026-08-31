import { getCurrentWebview } from '@tauri-apps/api/webview';
import { loadSettings, setSetting } from '$lib/api/db';
import type {
  AudioBitrate,
  RangeFocus,
  ReaderWidth,
  RepeatMode,
  Settings,
  Theme,
} from '$lib/types/database';

const APP_ZOOM_MIN = 0.7;
const APP_ZOOM_MAX = 1.5;
const READER_ZOOM_MIN = 0.7;
const READER_ZOOM_MAX = 2.0;
const ZOOM_STEP = 0.1;

/**
 * Quran text size, in CSS px before reader zoom.
 *
 * The two are separate on purpose and both are kept: this is the typographic
 * size the reader is set in — a preference, changed rarely, and the same in
 * both views — while reader zoom is a per-view magnifier with its own control
 * and its own remembered level for focus mode. Multiplying, rather than
 * replacing, is what lets a reader who has sized the text once still lean in.
 *
 * The ceiling is what the Mushaf page view can actually honour: its lines are
 * justified edge to edge and cannot reflow, so `.text-line` caps the rendered
 * size at 4.3cqi of the column. Past roughly 44px the scrolling view would
 * keep growing while the page view stopped, and the same setting would mean
 * two different things in the two views.
 */
export const FONT_SIZE_MIN = 18;
export const FONT_SIZE_MAX = 44;
export const FONT_SIZE_STEP = 1;

/** Leading, as a multiple of the font size. The floor is where the harakat of
 *  one line start colliding with the descenders of the line above. */
export const LINE_HEIGHT_MIN = 1.6;
export const LINE_HEIGHT_MAX = 3;
export const LINE_HEIGHT_STEP = 0.1;

/** The measure, in CSS px before reader zoom. Both reading views take their
 *  column width from this, so they stay the same width as each other. */
export const READER_WIDTHS: Record<ReaderWidth, string> = {
  narrow: '560px',
  normal: '720px',
  wide: '900px',
};

/** Repeats per verse. Two is barely a repeat; past twenty, the counter is the
 *  wrong control and the reader wants the loop left running. */
export const AUDIO_REPEAT_MIN = 1;
export const AUDIO_REPEAT_MAX = 20;
/** Longest silence offered between repetitions: enough to say back a long
 *  verse, not so long the player looks stuck. */
export const AUDIO_REPEAT_PAUSE_MAX = 10_000;
/** Recitation slowed for following along, or nudged up. Neither end is a
 *  speed anyone should read the Quran at for long, which is why the range is
 *  this narrow. */
export const AUDIO_RATE_MIN = 0.5;
export const AUDIO_RATE_MAX = 1.5;

function clampZoom(value: number, min: number, max: number): number {
  return Math.round(Math.min(max, Math.max(min, value)) * 10) / 10;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function defaultSettings(): Settings {
  return {
    theme: 'dark',
    font: 'amiri-quran',
    font_size: 28,
    line_height: 2.2,
    reader_width: 'normal',
    preferred_translation_id: null,
    show_translation: true,
    tafsir_id: null,
    show_tafsir: false,
    tafsir_panel_width: 420,
    tafsir_view: 'popover',
    tafsir_click: false,
    show_transliteration: false,
    show_ayah_numbers: true,
    range_focus: 'trim',
    app_zoom: 1,
    reader_zoom_normal: 1,
    reader_zoom_focus: 1,
    // No reciter until one is chosen, which is also the point at which audio
    // appears in the reader at all.
    reciter_id: null,
    audio_bitrate: 64,
    audio_repeat_mode: 'off',
    audio_repeat_count: 3,
    audio_repeat_pause_ms: 0,
    audio_playback_rate: 1,
    audio_follow: true,
    audio_volume: 1,
    audio_downloads_allowed: false,
  };
}

class SettingsStore {
  current = $state<Settings>(defaultSettings());
  ready = $state(false);

  async init() {
    if (this.ready) return;
    try {
      this.current = await loadSettings();
    } catch (err) {
      console.error('Failed to load settings, using defaults', err);
    }
    this.ready = true;
    this.applyTheme();
    this.applyTypography();
    this.applyAppZoom();
    // Focus mode always starts off on launch (it isn't persisted), so the reader opens
    // in its normal-view zoom.
    this.applyReaderZoom(false);
  }

  applyTheme() {
    if (typeof document === 'undefined') return;
    document.documentElement.dataset.theme = this.current.theme;
  }

  applyTypography() {
    if (typeof document === 'undefined') return;
    const root = document.documentElement.style;
    root.setProperty('--font-size-quran', `${this.current.font_size}px`);
    root.setProperty('--line-height-quran', `${this.current.line_height}`);
    root.setProperty(
      '--reader-max-width',
      READER_WIDTHS[this.current.reader_width] ?? READER_WIDTHS.normal,
    );
  }

  applyAppZoom() {
    if (typeof window === 'undefined') return;
    getCurrentWebview()
      .setZoom(this.current.app_zoom)
      .catch((err) => console.error('Failed to set webview zoom', err));
  }

  applyReaderZoom(focusMode: boolean) {
    if (typeof document === 'undefined') return;
    const zoom = focusMode ? this.current.reader_zoom_focus : this.current.reader_zoom_normal;
    document.documentElement.style.setProperty('--reader-zoom', `${zoom}`);
  }

  async setTheme(theme: Theme) {
    this.current.theme = theme;
    this.applyTheme();
    await setSetting('theme', theme);
  }

  /**
   * Quran text size. Rounded to whole pixels because that is the unit the
   * value is stored and displayed in, and a size the reader cannot type back
   * in is a size they cannot return to.
   */
  async setFontSize(px: number) {
    const value = Math.round(clamp(px, FONT_SIZE_MIN, FONT_SIZE_MAX));
    this.current.font_size = value;
    this.applyTypography();
    await setSetting('font_size', String(value));
  }

  /** One decimal place, for the same reason `setFontSize` rounds. */
  async setLineHeight(value: number) {
    const rounded = Math.round(clamp(value, LINE_HEIGHT_MIN, LINE_HEIGHT_MAX) * 10) / 10;
    this.current.line_height = rounded;
    this.applyTypography();
    await setSetting('line_height', String(rounded));
  }

  async setReaderWidth(width: ReaderWidth) {
    this.current.reader_width = width;
    this.applyTypography();
    await setSetting('reader_width', width);
  }

  /**
   * Whether the end-of-verse marker is rendered in the scrolling view.
   *
   * Scrolling view only, and that is not an oversight: the Mushaf page view
   * lays its lines out edge to edge from the printed page's own break points,
   * and the marker is one of the glyphs that line was justified around. Taking
   * it out would re-space the line and stop the page being the page.
   */
  async setShowAyahNumbers(show: boolean) {
    this.current.show_ayah_numbers = show;
    await setSetting('show_ayah_numbers', String(show));
  }

  /**
   * What the Mushaf page view does with the rest of the printed page.
   *
   * A printed page is shared: open Al-Mulk and the sheet it starts on carries
   * the last lines of Al-Mulk's predecessor above the banner, and the sheet it
   * ends on carries the opening of the next Surah below.
   *
   * Mushaf view only. The scrolling view is built from the range's own Ayahs
   * and has never had anything else in it to dim or drop.
   */
  async setRangeFocus(mode: RangeFocus) {
    this.current.range_focus = mode;
    await setSetting('range_focus', mode);
  }

  /** Back to the shipped typography in one action — three controls to put back
   *  by hand otherwise, and no way to tell when you have. */
  async resetTypography() {
    await this.setFontSize(28);
    await this.setLineHeight(2.2);
    await this.setReaderWidth('normal');
  }

  async setAppZoom(zoom: number) {
    this.current.app_zoom = clampZoom(zoom, APP_ZOOM_MIN, APP_ZOOM_MAX);
    this.applyAppZoom();
    await setSetting('app_zoom', String(this.current.app_zoom));
  }

  async setReaderZoom(zoom: number, focusMode: boolean) {
    const clamped = clampZoom(zoom, READER_ZOOM_MIN, READER_ZOOM_MAX);
    if (focusMode) {
      this.current.reader_zoom_focus = clamped;
    } else {
      this.current.reader_zoom_normal = clamped;
    }
    this.applyReaderZoom(focusMode);
    await setSetting(focusMode ? 'reader_zoom_focus' : 'reader_zoom_normal', String(clamped));
  }

  zoomAppIn() {
    return this.setAppZoom(this.current.app_zoom + ZOOM_STEP);
  }

  zoomAppOut() {
    return this.setAppZoom(this.current.app_zoom - ZOOM_STEP);
  }

  resetAppZoom() {
    return this.setAppZoom(1);
  }

  zoomReaderIn(focusMode: boolean) {
    const current = focusMode ? this.current.reader_zoom_focus : this.current.reader_zoom_normal;
    return this.setReaderZoom(current + ZOOM_STEP, focusMode);
  }

  zoomReaderOut(focusMode: boolean) {
    const current = focusMode ? this.current.reader_zoom_focus : this.current.reader_zoom_normal;
    return this.setReaderZoom(current - ZOOM_STEP, focusMode);
  }

  resetReaderZoom(focusMode: boolean) {
    return this.setReaderZoom(1, focusMode);
  }

  // ===========================================================================
  // AUDIO
  // ===========================================================================

  /**
   * Choose a reciter, or `null` for none.
   *
   * This is the setting that makes audio exist at all: with no reciter the bar
   * never appears and nothing is ever fetched. There is deliberately no default
   * — see docs/audio-plan.md, "Privacy".
   */
  async setReciter(id: number | null) {
    this.current.reciter_id = id;
    await setSetting('reciter_id', id === null ? '' : String(id));

    // Choosing a reciter *is* the consent to fetch, and this is the only place
    // it can be chosen — a section that states plainly where recitation comes
    // from and what it costs. Asking again at the first play would be asking
    // the same question twice, in a worse place for it. The toggle stays, for
    // turning fetching back off ("cached only") afterwards.
    if (id !== null && !this.current.audio_downloads_allowed) {
      await this.setAudioDownloadsAllowed(true);
    }
  }

  /** 64 or 128. The cache is keyed by bitrate, so switching starts a second
   *  cache rather than replacing the first. */
  async setAudioBitrate(bitrate: AudioBitrate) {
    this.current.audio_bitrate = bitrate;
    await setSetting('audio_bitrate', String(bitrate));
  }

  async setAudioRepeatMode(mode: RepeatMode) {
    this.current.audio_repeat_mode = mode;
    await setSetting('audio_repeat_mode', mode);
  }

  async setAudioRepeatCount(count: number) {
    const value = Math.round(clamp(count, AUDIO_REPEAT_MIN, AUDIO_REPEAT_MAX));
    this.current.audio_repeat_count = value;
    await setSetting('audio_repeat_count', String(value));
  }

  /** Silence between repetitions, in milliseconds. The point of it is to leave
   *  room to say the verse back before it comes round again. */
  async setAudioRepeatPause(ms: number) {
    const value = Math.round(clamp(ms, 0, AUDIO_REPEAT_PAUSE_MAX));
    this.current.audio_repeat_pause_ms = value;
    await setSetting('audio_repeat_pause_ms', String(value));
  }

  async setAudioPlaybackRate(rate: number) {
    const value = Math.round(clamp(rate, AUDIO_RATE_MIN, AUDIO_RATE_MAX) * 100) / 100;
    this.current.audio_playback_rate = value;
    await setSetting('audio_playback_rate', String(value));
  }

  async setAudioFollow(follow: boolean) {
    this.current.audio_follow = follow;
    await setSetting('audio_follow', String(follow));
  }

  async setAudioVolume(volume: number) {
    const value = Math.round(clamp(volume, 0, 1) * 100) / 100;
    this.current.audio_volume = value;
    await setSetting('audio_volume', String(value));
  }

  /**
   * The network switch. Off means cached verses still play and nothing leaves
   * the machine, which is what someone who downloaded a Juz for a journey
   * actually wants.
   */
  async setAudioDownloadsAllowed(allowed: boolean) {
    this.current.audio_downloads_allowed = allowed;
    await setSetting('audio_downloads_allowed', String(allowed));
  }
}

export const settingsStore = new SettingsStore();
