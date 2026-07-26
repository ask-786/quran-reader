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

  let { children } = $props();

  // n/p step through the adjacent Surah/Juz/Hizb, bounds keyed by the current route.
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
    }
  }

  onMount(() => {
    settingsStore.init();
    surahsStore.init();
    bookmarksStore.init();

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

      if (uiStore.goToOpen || e.ctrlKey || e.metaKey || e.altKey) return;
      const target = e.target as HTMLElement;
      if (target.isContentEditable || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        return;
      }

      switch (e.key.toLowerCase()) {
        case 'f':
          uiStore.toggleFocusMode();
          break;
        case 'n':
          navigateAdjacent(1);
          break;
        case 'p':
          navigateAdjacent(-1);
          break;
      }
    }
    window.addEventListener('keydown', onKeydown);
    return () => window.removeEventListener('keydown', onKeydown);
  });
</script>

<div class="app-shell">
  {#if !uiStore.focusMode}
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
