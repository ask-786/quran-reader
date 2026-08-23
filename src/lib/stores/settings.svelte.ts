import { getCurrentWebview } from '@tauri-apps/api/webview';
import { loadSettings, setSetting } from '$lib/api/db';
import type { Settings, Theme } from '$lib/types/database';

const APP_ZOOM_MIN = 0.7;
const APP_ZOOM_MAX = 1.5;
const READER_ZOOM_MIN = 0.7;
const READER_ZOOM_MAX = 2.0;
const ZOOM_STEP = 0.1;

function clampZoom(value: number, min: number, max: number): number {
  return Math.round(Math.min(max, Math.max(min, value)) * 10) / 10;
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
    show_transliteration: false,
    show_ayah_numbers: true,
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
    const widths = { narrow: '560px', normal: '720px', wide: '900px' };
    root.setProperty('--reader-max-width', widths[this.current.reader_width] ?? widths.normal);
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
