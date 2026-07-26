<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import GoToAyahDialog from '$lib/components/navigation/GoToAyahDialog.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let { children } = $props();

  onMount(() => {
    settingsStore.init();
    surahsStore.init();
    bookmarksStore.init();

    function onKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (uiStore.goToAyahOpen) {
          uiStore.closeGoToAyah();
        } else if (uiStore.focusMode) {
          uiStore.exitFocusMode();
        }
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'g') {
        e.preventDefault();
        uiStore.toggleGoToAyah();
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

<GoToAyahDialog />

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
