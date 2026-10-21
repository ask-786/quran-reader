import { getCurrentWindow } from '@tauri-apps/api/window';
import { isMac } from '$lib/utils/platform';
import { uiStore } from './ui.svelte';

/**
 * macOS gets the native title bar, laid over the top of the webview
 * (`tauri.macos.conf.json`), instead of the borderless window and drawn
 * controls the other platforms use. A borderless NSWindow has no zoom button
 * and no full-screen behaviour of its own: maximising it only filled the space
 * between the menu bar and the Dock, and nothing could take it full screen.
 */
class WindowChromeStore {
  /**
   * Native full screen. macOS hides the traffic lights with the menu bar
   * there; elsewhere it is F11 that gets the window into it.
   */
  fullscreen = $state(false);

  /**
   * Whether the traffic lights are on screen, over the top-left corner of the
   * window. Whatever is drawn there — the sidebar's head, or the toolbar with
   * the sidebar shut — has to leave room for them.
   */
  get trafficLights() {
    return isMac && !this.fullscreen;
  }

  /**
   * Whether focus mode is what took the window full screen, and so whether
   * leaving it should bring the window back. A window that was already full
   * screen stays that way.
   */
  #fullscreenForFocus = false;

  /** Track full screen. Returns the cleanup. */
  init(): () => void {
    const appWindow = getCurrentWindow();
    const sync = async () => {
      const was = this.fullscreen;
      this.fullscreen = await appWindow.isFullscreen();
      // Left by the green button or ⌃⌘F rather than by leaving focus mode,
      // which clears the flag first. Focus mode can't stay on without full
      // screen, or the traffic lights land over the reader.
      if (was && !this.fullscreen && this.#fullscreenForFocus) {
        this.#fullscreenForFocus = false;
        uiStore.exitFocusMode();
      }
    };
    void sync();
    // Entering and leaving full screen both resize the window, so this is the
    // only event needed.
    const unlisten = appWindow.onResized(sync);
    return () => {
      unlisten.then((fn) => fn());
    };
  }

  /**
   * F11, for Windows and Linux, where maximising keeps the taskbar or panel on
   * screen. macOS has its own key for this (Ctrl+Cmd+F, from the View menu
   * Tauri installs) and the green button, and gives F11 to the system.
   */
  async toggleFullscreen() {
    if (isMac) return;
    await getCurrentWindow().setFullscreen(!this.fullscreen);
  }

  /**
   * On macOS focus mode is full screen too. It hides the toolbar and sidebar,
   * the two things that make room for the traffic lights, and full screen is
   * the one place macOS takes them away.
   */
  async followFocusMode(on: boolean) {
    if (!isMac) return;
    if (on && !this.fullscreen) {
      this.#fullscreenForFocus = true;
      await getCurrentWindow().setFullscreen(true);
    } else if (!on && this.#fullscreenForFocus) {
      this.#fullscreenForFocus = false;
      await getCurrentWindow().setFullscreen(false);
    }
  }
}

export const windowChrome = new WindowChromeStore();
