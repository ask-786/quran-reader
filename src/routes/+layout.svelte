<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { X } from 'lucide-svelte';
  import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import ReaderZoomControl from '$lib/components/reader/ReaderZoomControl.svelte';
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
      if (e.key === 'Escape' && uiStore.focusMode) uiStore.exitFocusMode();
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

  {#if uiStore.focusMode}
    <div class="focus-overlay">
      <ReaderZoomControl />
      <button class="exit-btn" onclick={() => uiStore.exitFocusMode()} aria-label="Exit focus mode">
        <X size={16} />
      </button>
    </div>
  {/if}
</div>

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

  .focus-overlay {
    position: absolute;
    top: 12px;
    right: 16px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px;
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25);
    z-index: 10;
  }

  .exit-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: background var(--transition);
  }

  .exit-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }
</style>
