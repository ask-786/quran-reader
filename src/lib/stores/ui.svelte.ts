import { settingsStore } from './settings.svelte';

export type ReadingMode = 'scroll' | 'mushaf';

class UiStore {
  sidebarOpen = $state(true);
  readingMode = $state<ReadingMode>('mushaf');
  focusMode = $state(false);
  goToOpen = $state(false);

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

  openGoTo() {
    this.goToOpen = true;
  }

  closeGoTo() {
    this.goToOpen = false;
  }

  toggleGoTo() {
    this.goToOpen = !this.goToOpen;
  }
}

export const uiStore = new UiStore();
