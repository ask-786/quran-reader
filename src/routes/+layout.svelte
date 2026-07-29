<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import GoToDialog from '$lib/components/navigation/GoToDialog.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';
  import { isNarrowViewport } from '$lib/utils/viewport';

  const MAX_PAGE = 604;

  let { children } = $props();

  // n/p step through the adjacent Surah/Juz/Hizb/page, bounds keyed by the current route.
  function navigateAdjacent(direction: 1 | -1) {
    const currentId = Number($page.params.id);
    if (!Number.isInteger(currentId)) return;
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

  onMount(() => {
    settingsStore.init();
    surahsStore.init();
    bookmarksStore.init();

    // On a phone-sized window the sidebar starts as a closed overlay rather
    // than a docked column stealing half the screen.
    if (isNarrowViewport()) uiStore.sidebarOpen = false;

    function onKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (uiStore.goToOpen) {
          uiStore.closeGoTo();
        } else if (uiStore.focusMode) {
          uiStore.exitFocusMode();
        }
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'g') {
        e.preventDefault();
        uiStore.toggleGoTo();
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

      if (uiStore.goToOpen || e.ctrlKey || e.metaKey || e.altKey) return;
      const target = e.target as HTMLElement;
      if (target.isContentEditable || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
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
          navigateAdjacent(1);
          break;
        case 'p':
          navigateAdjacent(-1);
          break;
        case 'a':
          autoScrollStore.toggle();
          break;
      }
    }
    window.addEventListener('keydown', onKeydown);
    return () => window.removeEventListener('keydown', onKeydown);
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

<GoToDialog />

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
