<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let { children } = $props();

  onMount(() => {
    settingsStore.init();
    surahsStore.init();
    bookmarksStore.init();
  });
</script>

<div class="app-shell">
  <div class="sidebar-slot" class:collapsed={!uiStore.sidebarOpen}>
    <Sidebar />
  </div>
  <div class="main-column">
    <Toolbar />
    <div class="page-slot">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .app-shell {
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
