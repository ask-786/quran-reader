class UiStore {
  sidebarOpen = $state(true);

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen;
  }
}

export const uiStore = new UiStore();
