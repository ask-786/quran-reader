export type ReadingMode = 'scroll' | 'mushaf';

class UiStore {
  sidebarOpen = $state(true);
  readingMode = $state<ReadingMode>('mushaf');
  focusMode = $state(false);
  goToAyahOpen = $state(false);

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

  openGoToAyah() {
    this.goToAyahOpen = true;
  }

  closeGoToAyah() {
    this.goToAyahOpen = false;
  }

  toggleGoToAyah() {
    this.goToAyahOpen = !this.goToAyahOpen;
  }
}

export const uiStore = new UiStore();
