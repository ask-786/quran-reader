import { loadSettings, setSetting } from '$lib/api/db';
import type { Settings, Theme } from '$lib/types/database';

function defaultSettings(): Settings {
  return {
    theme: 'dark',
    font: 'amiri-quran',
    font_size: 28,
    line_height: 2.2,
    reader_width: 'normal',
    last_read_surah_id: 1,
    last_read_ayah_id: 1,
    preferred_translation_id: null,
    show_translation: true,
    show_transliteration: false,
    show_ayah_numbers: true,
    scroll_position: 0,
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

  async setTheme(theme: Theme) {
    this.current.theme = theme;
    this.applyTheme();
    await setSetting('theme', theme);
  }

  async setLastRead(surahId: number, ayahId: number) {
    this.current.last_read_surah_id = surahId;
    this.current.last_read_ayah_id = ayahId;
    await Promise.all([
      setSetting('last_read_surah_id', String(surahId)),
      setSetting('last_read_ayah_id', String(ayahId)),
    ]);
  }
}

export const settingsStore = new SettingsStore();
