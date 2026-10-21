/**
 * `navigator.platform` is deprecated but still the one field WKWebView fills
 * in reliably, and the app only ever runs in Tauri's webview.
 */
export const isMac = /Mac/.test(navigator.platform);

/**
 * A modifier shortcut as the platform writes it: `⌘K` on a Mac, `Ctrl+K`
 * everywhere else. The handlers take either modifier on every platform; this
 * is only the label.
 */
export function modShortcut(key: string): string {
  return isMac ? `⌘${key}` : `Ctrl+${key}`;
}
