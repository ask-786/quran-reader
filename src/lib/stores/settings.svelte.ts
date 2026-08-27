import { getCurrentWebview } from '@tauri-apps/api/webview';
import { loadSettings, setSetting } from '$lib/api/db';
import type { RangeFocus, ReaderWidth, Settings, Theme } from '$lib/types/database';

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
}

export const settingsStore = new SettingsStore();
