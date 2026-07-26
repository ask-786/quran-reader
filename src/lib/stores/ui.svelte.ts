import { settingsStore } from './settings.svelte';

export type ReadingMode = 'scroll' | 'mushaf';

class UiStore {
  sidebarOpen = $state(true);
  readingMode = $state<ReadingMode>('mushaf');
  focusMode = $state(false);
  goToOpen = $state(false);

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
