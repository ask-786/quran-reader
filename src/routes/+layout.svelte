<script lang="ts">
  import '../app.css';
  import { onMount, untrack } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import NavPalette from '$lib/components/navigation/NavPalette.svelte';
  import SettingsDialog from '$lib/components/settings/SettingsDialog.svelte';
  import RoutineEditor from '$lib/components/routines/RoutineEditor.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';
  import { readingStore } from '$lib/stores/reading.svelte';
  import { routinesStore } from '$lib/stores/routines.svelte';
  import { windowChrome } from '$lib/stores/window-chrome.svelte';
  import { isNarrowViewport } from '$lib/utils/viewport';
  import { isMac } from '$lib/utils/platform';

  const MAX_PAGE = 604;

  let { children } = $props();

  /**
   * n/p step through the adjacent Surah/Juz/Hizb/page, bounds keyed by the
   * current route — or, on the Surah a routine is up to, through the routine.
   *
   * `plain` is Shift+N/P: the Mushaf's own neighbour even inside a routine, the
   * deliberate way out of it. The routine isn't ended by that, only left; the
   * toolbar chip is the way back.
   */
  function navigateAdjacent(direction: 1 | -1, plain = false) {
    const currentId = Number($page.params.id);
    if (!Number.isInteger(currentId)) return;

    if (!plain && routinesStore.isOnStep($page.route.id, currentId)) {
      const href = routinesStore.stepHref(direction);
      // Past either end of a routine is nowhere — falling through to the
      // Mushaf's next Surah would quietly carry the reader out of it. Forward
      // off the last step is finishing it.
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- built by routineStepHref, which resolves it
      if (href) goto(href);
      else if (direction === 1) void routinesStore.complete();
      return;
    }

    const next = currentId + direction;

    switch ($page.route.id) {
      case '/surah/[id]':
        if (next >= 1 && next <= 114) goto(resolve('/surah/[id]', { id: String(next) }));
        break;
      case '/juz/[id]':
        if (next >= 1 && next <= 30) goto(resolve('/juz/[id]', { id: String(next) }));
        break;
      case '/hizb/[id]':
        if (next >= 1 && next <= 60) goto(resolve('/hizb/[id]', { id: String(next) }));
        break;
      case '/page/[id]':
        if (next >= 1 && next <= MAX_PAGE) goto(resolve('/page/[id]', { id: String(next) }));
        break;
    }
  }

  /**
   * Step one Mushaf page. Inside a Surah/Juz/Hizb that's a scroll to the next
   * page boundary of the open range; on the single-page route the range is one
   * page, so the same key has to move to the next page of the Mushaf instead.
   */
  function stepPage(direction: 1 | -1) {
    if ($page.route.id === '/page/[id]') {
      navigateAdjacent(direction);
      return;
    }
    readerScroll.jumpPage(direction);
  }

  /**
   * The macOS menu bar's items (src-tauri/src/menu.rs), by id. Each does what
   * its key binding below does, except that the menu's Settings… and Go To…
   * only ever open: a menu item names a place, not a switch.
   */
  const MENU_ITEMS: Record<string, () => void> = {
    settings: () => uiStore.openSettings(),
    find: () => {
      uiStore.closePalette();
      uiStore.focusSearch();
    },
    'toggle-sidebar': () => uiStore.toggleSidebar(),
    'reading-mode': () => uiStore.toggleReadingMode(),
    'verse-cards': () => tafsirStore.toggle(),
    'focus-mode': () => uiStore.toggleFocusMode(),
    'zoom-in': () => void settingsStore.zoomAppIn(),
    'zoom-out': () => void settingsStore.zoomAppOut(),
    'zoom-reset': () => void settingsStore.resetAppZoom(),
    'go-to': () => uiStore.openPalette(),
    next: () => navigateAdjacent(1),
    previous: () => navigateAdjacent(-1),
    routines: () => uiStore.toggleRoutinesMenu(),
  };

  /**
   * The items whose shortcut the keydown handler takes too. WebKit gives the
   * page the first look at a key and passes it to the menu only when the page
   * leaves it alone — which these all preventDefault — so the menu shouldn't
   * see them. If it ever does, running both would toggle the palette open and
   * shut again in one keystroke; a menu event for one of these hard on the
   * heels of a modifier keydown is that same keystroke, and is dropped.
   */
  const KEYED_MENU_ITEMS = new Set([
    'settings',
    'find',
    'zoom-in',
    'zoom-out',
    'zoom-reset',
    'go-to',
  ]);
  let modifierKeydownAt = -Infinity;

  function runMenuItem(id: string) {
    if (KEYED_MENU_ITEMS.has(id) && performance.now() - modifierKeydownAt < 500) return;
    MENU_ITEMS[id]?.();
  }

  // Only focus mode is the trigger; the full-screen state the call reads is
  // not, or the window coming out of full screen would re-run it.
  $effect(() => {
    const on = uiStore.focusMode;
    untrack(() => void windowChrome.followFocusMode(on));
  });

  onMount(() => {
    // Tafsir state (chosen edition, panel width, whether it is open) lives in
    // the settings row, so its store can only read it once settings have
    // landed — hence the chain rather than another fire-and-forget init.
    settingsStore.init().then(() => tafsirStore.init());
    surahsStore.init();
    bookmarksStore.init();
    routinesStore.init();
    // The reciter catalogue. A local query that makes no request of its own;
    // the tafsir card's listen button needs it to know whether a reciter has
    // been chosen.
    playbackStore.init();
    const stopWindowChrome = windowChrome.init();
    const unlistenMenu = isMac ? listen<string>('menu', (e) => runMenuItem(e.payload)) : null;

    // On a phone-sized window the sidebar starts as a closed overlay rather
    // than a docked column stealing half the screen.
    if (isNarrowViewport()) uiStore.sidebarOpen = false;

    // The position is written a beat after the reader settles, so closing the
    // window (or hiding it, which is as close to a warning as a backgrounded
    // app gets) has to bank whatever is still waiting.
    function flushReadingPosition() {
      if (document.visibilityState === 'hidden') readingStore.flush();
    }
    window.addEventListener('beforeunload', () => readingStore.flush());
    document.addEventListener('visibilitychange', flushReadingPosition);

    /**
     * One dismissal rule for both tafsir surfaces — see
     * `tafsirStore.notePointerDown`. Registered here rather than inside the
     * card and the panel because it has to hold whichever of them is mounted,
     * and because a click on a verse has to reach the store before that verse's
     * own handler can reopen what this just closed.
     *
     * Capture, so it lands before the click can reach whatever it was aimed at.
     * Suspended under a modal: Settings and the palette cover the panel rather
     * than sitting beside it, so a click in either is not a click away from it
     * — and dismissing the settings dialog by its backdrop would otherwise take
     * the panel down too.
     */
    function onPointerDown(e: PointerEvent) {
      if (uiStore.modalOpen) return;
      const target = e.target;
      tafsirStore.notePointerDown(target instanceof Element ? target : null);
    }
    window.addEventListener('pointerdown', onPointerDown, true);

    function onKeydown(e: KeyboardEvent) {
      if (e.ctrlKey || e.metaKey) modifierKeydownAt = performance.now();
      if (e.key === 'Escape') {
        // Outermost first. Settings and the palette never coexist (each closes
        // the other), so their order between themselves is academic — but both
        // sit over the tafsir popover. The palette clears its own filter on the
        // first Escape and only lets the key reach here when there is nothing
        // left to clear, so it goes before the popover beneath it — and the
        // popover before focus mode, or dismissing a card in focus mode would
        // tear down the whole chrome instead.
        if (uiStore.routineEditor) {
          uiStore.closeRoutineEditor();
        } else if (uiStore.routinesMenuOpen) {
          uiStore.closeRoutinesMenu();
        } else if (uiStore.settingsOpen) {
          uiStore.closeSettings();
        } else if (uiStore.paletteOpen) {
          uiStore.closePalette();
        } else if (tafsirStore.selection) {
          tafsirStore.closePopover();
        } else if (uiStore.focusMode) {
          uiStore.exitFocusMode();
        }
        return;
      }
      // Full screen, from anywhere — a text field included, as in a browser.
      if (e.key === 'F11') {
        e.preventDefault();
        void windowChrome.toggleFullscreen();
        return;
      }
      // The palette's own keys. Ctrl/Cmd+K is the one everything else uses,
      // alongside the older Ctrl/Cmd+G. Both toggle, so the same keystroke puts
      // it away. `/` and Ctrl/Cmd+F below stay pointed at the sidebar: the two
      // surfaces list the same things, and which one you get should be your
      // choice, not a side effect of which search key fell under your hand.
      if (
        (e.ctrlKey || e.metaKey) &&
        (e.key.toLowerCase() === 'k' || e.key.toLowerCase() === 'g')
      ) {
        e.preventDefault();
        uiStore.togglePalette();
        return;
      }
      // Ctrl/Cmd+comma, the platform-standard preferences key. Toggles, so the
      // same keystroke puts it away.
      if ((e.ctrlKey || e.metaKey) && e.key === ',') {
        e.preventDefault();
        uiStore.toggleSettings();
        return;
      }

      // The platform-standard find key, pointed at the sidebar's filter box.
      // Unlike `/` below it works from inside a text field too, so the palette
      // has to give way to it.
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
        e.preventDefault();
        uiStore.closePalette();
        uiStore.focusSearch();
        return;
      }

      // App zoom takes the platform-standard Ctrl/Cmd +/-/0, so it has to be
      // handled ahead of the modifier guard below. `=` and `_` are the
      // unshifted faces of `+` and `-`, accepted so the binding works whether
      // or not Shift is held.
      if ((e.ctrlKey || e.metaKey) && !e.altKey) {
        switch (e.key) {
          case '+':
          case '=':
            e.preventDefault();
            void settingsStore.zoomAppIn();
            return;
          case '-':
          case '_':
            e.preventDefault();
            void settingsStore.zoomAppOut();
            return;
          case '0':
            e.preventDefault();
            void settingsStore.resetAppZoom();
            return;
        }
      }

      if (uiStore.modalOpen || e.ctrlKey || e.metaKey || e.altKey) {
        return;
      }
      const target = e.target as HTMLElement;
      if (target.isContentEditable || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        return;
      }

      // Bare `/`, the vim/browser convention. Safe as an unmodified key because
      // the text-field guard above has already bowed out of any input.
      if (e.key === '/') {
        e.preventDefault();
        uiStore.focusSearch();
        return;
      }

      // Space means play/pause while a verse card is open, and the auto-scroll
      // toggle the rest of the time.
      //
      // Which is not two bindings but one: Space is "start or stop the thing
      // that is moving". With a card open, the thing that moves is the
      // recitation of that verse; with the reader alone on screen, it is the
      // page. The card is a deliberate, visible state, so which one the key
      // means is never a guess.
      //
      // It has to preventDefault or the browser scrolls the reader a page down
      // on top of whichever it did, and it steps aside for a focused control,
      // where Space is that control's own activation key.
      if (e.key === ' ') {
        if (
          target instanceof Element &&
          target.closest('button, a[href], select, [role="button"], [role="option"]')
        ) {
          return;
        }
        e.preventDefault();
        // No reciter, nothing to play through, or a panel open on nothing falls
        // through to auto-scroll rather than doing nothing at all — the key
        // keeps a meaning either way.
        const cardAyahId = tafsirStore.anyOpen ? tafsirStore.targetAyahId : null;
        if (cardAyahId !== null && playbackStore.enabled && playbackStore.playable) {
          void playbackStore.toggle(cardAyahId);
        } else {
          autoScrollStore.toggle();
        }
        return;
      }

      // Shift+arrows tune the auto-scroll speed — the same nudge the handle's
      // own arrow keys give it. Bare arrows page the Mushaf, below.
      if (e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
        e.preventDefault();
        if (e.key === 'ArrowUp') autoScrollStore.faster();
        else autoScrollStore.slower();
        return;
      }

      // Paging and the jump-to-end keys move the reader — but only when the
      // reader is what you're in. With a Surah link in the sidebar focused,
      // these same keys scroll that list natively, and taking them over there
      // would leave the list with no keyboard scrolling at all.
      // `instanceof Element` because a keydown with nothing focused can be
      // targeted at the document itself, which has no closest().
      if (!(target instanceof Element) || !target.closest('.sidebar-slot')) {
        switch (e.key) {
          case 'ArrowDown':
          case 'PageDown':
            e.preventDefault();
            stepPage(1);
            return;
          case 'ArrowUp':
          case 'PageUp':
            e.preventDefault();
            stepPage(-1);
            return;
          case 'Home':
            e.preventDefault();
            readerScroll.scrollToTop();
            return;
          case 'End':
            e.preventDefault();
            readerScroll.scrollToBottom();
            return;
        }
      }

      switch (e.key) {
        // Reader zoom, mirroring the app-zoom bindings above without the
        // modifier. Which of the two remembered levels this moves depends on
        // focus mode, exactly as the on-screen control does.
        case '+':
        case '=':
          e.preventDefault();
          void settingsStore.zoomReaderIn(uiStore.focusMode);
          return;
        case '-':
        case '_':
          e.preventDefault();
          void settingsStore.zoomReaderOut(uiStore.focusMode);
          return;
        case '0':
          e.preventDefault();
          void settingsStore.resetReaderZoom(uiStore.focusMode);
          return;
      }

      switch (e.key.toLowerCase()) {
        case 'f':
          uiStore.toggleFocusMode();
          break;
        case 'm':
          uiStore.toggleReadingMode();
          break;
        case 'n':
          navigateAdjacent(1, e.shiftKey);
          break;
        case 'p':
          navigateAdjacent(-1, e.shiftKey);
          break;
        case 'r':
          uiStore.toggleRoutinesMenu();
          break;
        case 'a':
          autoScrollStore.toggle();
          break;
        case 't':
          tafsirStore.toggle();
          break;
      }
    }
    window.addEventListener('keydown', onKeydown);
    return () => {
      stopWindowChrome();
      unlistenMenu?.then((fn) => fn());
      window.removeEventListener('keydown', onKeydown);
      window.removeEventListener('pointerdown', onPointerDown, true);
      document.removeEventListener('visibilitychange', flushReadingPosition);
    };
  });
</script>

<div class="app-shell">
  {#if !uiStore.focusMode}
    <div
      class="sidebar-backdrop"
      class:visible={uiStore.sidebarOpen}
      role="presentation"
      onclick={() => uiStore.toggleSidebar()}
    ></div>
    <div class="sidebar-slot" class:collapsed={!uiStore.sidebarOpen}>
      <Sidebar />
    </div>
  {/if}
  <div class="main-column">
    {#if !uiStore.focusMode}
      <Toolbar />
    {/if}
    <div class="page-slot">
      {@render children()}
    </div>
  </div>
</div>

<NavPalette />
<SettingsDialog />
<RoutineEditor />

<style>
  .app-shell {
    position: relative;
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .sidebar-slot {
    flex-shrink: 0;
    width: var(--sidebar-width);
    overflow: hidden;
    transition: width var(--transition);
  }

  .sidebar-slot.collapsed {
    width: 0;
  }

  .sidebar-backdrop {
    display: none;
  }

  /* Below the tablet breakpoint the sidebar docks on top of the content as a
     sliding drawer instead of sharing the row with it. */
  @media (max-width: 900px) {
    .sidebar-slot {
      position: fixed;
      inset: 0 auto 0 0;
      z-index: 50;
      width: min(var(--sidebar-width), 85vw);
      transform: translateX(0);
      box-shadow: 4px 0 24px rgba(0, 0, 0, 0.3);
    }

    .sidebar-slot.collapsed {
      width: min(var(--sidebar-width), 85vw);
      transform: translateX(-100%);
    }

    .sidebar-backdrop.visible {
      display: block;
      position: fixed;
      inset: 0;
      z-index: 49;
      background: rgba(0, 0, 0, 0.4);
    }
  }

  .main-column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .page-slot {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
