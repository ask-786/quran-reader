import { settingsStore } from './settings.svelte';

export type ReadingMode = 'scroll' | 'mushaf';

class UiStore {
  sidebarOpen = $state(true);
  readingMode = $state<ReadingMode>('mushaf');
  focusMode = $state(false);
  /**
   * The command palette — the search box, the Surah/Juz/Hizb/Recent lists, and
   * the `2:255` jump, over the reader. It is the keyboard route into all of it,
   * which is why it works in focus mode and with the sidebar shut instead of
   * dragging either of them back on screen.
   */
  paletteOpen = $state(false);

  /**
   * Set by the search shortcut, cleared by the sidebar once it has focused its
   * filter box. A flag rather than a counter so a later sidebar remount — a
   * focus-mode toggle, say — doesn't replay a focus the user never asked for.
   */
  searchFocusPending = $state(false);

  /**
   * The reader zoom in force right now. Normal and focus view remember separate
   * levels, so this changes on a focus toggle without any zoom control being
   * touched — which is why the reader views watch it to hold their position.
   */
  get readerZoom() {
    return this.focusMode
      ? settingsStore.current.reader_zoom_focus
      : settingsStore.current.reader_zoom_normal;
  }

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen;
  }

  toggleReadingMode() {
    this.readingMode = this.readingMode === 'scroll' ? 'mushaf' : 'scroll';
  }

  toggleFocusMode() {
    this.focusMode = !this.focusMode;
    // Normal and focus view remember separate reader zoom levels.
    settingsStore.applyReaderZoom(this.focusMode);
  }

  exitFocusMode() {
    this.focusMode = false;
    settingsStore.applyReaderZoom(false);
  }

  openPalette() {
    this.paletteOpen = true;
  }

  closePalette() {
    this.paletteOpen = false;
  }

  togglePalette() {
    this.paletteOpen = !this.paletteOpen;
  }

  /**
   * Put the caret in the sidebar's filter box. The sidebar has to be on screen
   * for that, so a collapsed sidebar opens and focus mode ends first.
   */
  focusSearch() {
    if (this.focusMode) this.exitFocusMode();
    this.sidebarOpen = true;
    this.searchFocusPending = true;
  }
}

export const uiStore = new UiStore();
