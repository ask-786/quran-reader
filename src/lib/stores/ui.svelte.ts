export type ReadingMode = 'scroll' | 'mushaf';

class UiStore {
  sidebarOpen = $state(true);
  readingMode = $state<ReadingMode>('mushaf');
  focusMode = $state(false);

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen;
  }

  toggleReadingMode() {
    this.readingMode = this.readingMode === 'scroll' ? 'mushaf' : 'scroll';
  }

  toggleFocusMode() {
    this.focusMode = !this.focusMode;
  }

  exitFocusMode() {
    this.focusMode = false;
  }
}

export const uiStore = new UiStore();
