import { getCurrentWindow } from '@tauri-apps/api/window';
import { isMac } from '$lib/utils/platform';

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

  /** Track full screen. Returns the cleanup. */
  init(): () => void {
    const appWindow = getCurrentWindow();
    const sync = async () => {
      this.fullscreen = await appWindow.isFullscreen();
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
}

export const windowChrome = new WindowChromeStore();
