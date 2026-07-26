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
  }

  exitFocusMode() {
    this.focusMode = false;
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
