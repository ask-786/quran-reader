<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import {
    PanelLeft,
    Moon,
    Sun,
    Coffee,
    BookOpen,
    Rows3,
    Minus,
    Square,
    Copy,
    X,
    ZoomIn,
    ZoomOut,
    Focus,
  } from 'lucide-svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Theme } from '$lib/types/database';

  const THEMES: Theme[] = ['dark', 'light', 'sepia'];
  const appWindow = getCurrentWindow();

  const surah = $derived(
    $page.data?.surah as { name_en: string; transliteration: string } | undefined,
  );

  let isMaximized = $state(false);

  onMount(() => {
    appWindow.isMaximized().then((v) => (isMaximized = v));
    const unlisten = appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  function cycleTheme() {
    const i = THEMES.indexOf(settingsStore.current.theme);
    settingsStore.setTheme(THEMES[(i + 1) % THEMES.length]);
  }
</script>

<header class="toolbar" data-tauri-drag-region>
  <button class="icon-btn" onclick={() => uiStore.toggleSidebar()} aria-label="Toggle sidebar">
    <PanelLeft size={18} />
  </button>

  <h1 class="title">{surah ? surah.transliteration : 'Quran Reader'}</h1>

  {#if surah}
    <button
      class="icon-btn"
      onclick={() => uiStore.toggleReadingMode()}
      aria-label={uiStore.readingMode === 'scroll'
        ? 'Switch to Mushaf page view'
        : 'Switch to scrolling view'}
      title={uiStore.readingMode === 'scroll' ? 'Mushaf page view' : 'Scrolling view'}
    >
      {#if uiStore.readingMode === 'scroll'}
        <BookOpen size={18} />
      {:else}
        <Rows3 size={18} />
      {/if}
    </button>
  {/if}

  <div class="zoom-group" title="App zoom">
    <button class="icon-btn small" onclick={() => settingsStore.zoomAppOut()} aria-label="Zoom out">
      <ZoomOut size={15} />
    </button>
    <button
      class="zoom-value"
      onclick={() => settingsStore.resetAppZoom()}
      aria-label="Reset app zoom"
    >
      {Math.round(settingsStore.current.app_zoom * 100)}%
    </button>
    <button class="icon-btn small" onclick={() => settingsStore.zoomAppIn()} aria-label="Zoom in">
      <ZoomIn size={15} />
    </button>
  </div>

  {#if surah}
    <button
      class="icon-btn"
      onclick={() => uiStore.toggleFocusMode()}
      aria-label="Enter focus mode"
      title="Focus mode"
    >
      <Focus size={18} />
    </button>
  {/if}

  <button
    class="icon-btn"
    onclick={cycleTheme}
    aria-label="Change theme ({settingsStore.current.theme})"
  >
    {#if settingsStore.current.theme === 'dark'}
      <Moon size={18} />
    {:else if settingsStore.current.theme === 'light'}
      <Sun size={18} />
    {:else}
      <Coffee size={18} />
    {/if}
  </button>

  <div class="window-controls">
    <button class="icon-btn" onclick={() => appWindow.minimize()} aria-label="Minimize window">
      <Minus size={16} />
    </button>
    <button
      class="icon-btn"
      onclick={() => appWindow.toggleMaximize()}
      aria-label={isMaximized ? 'Restore window' : 'Maximize window'}
    >
      {#if isMaximized}
        <Copy size={14} />
      {:else}
        <Square size={14} />
      {/if}
    </button>
    <button class="icon-btn close-btn" onclick={() => appWindow.close()} aria-label="Close window">
      <X size={16} />
    </button>
  </div>
</header>

<style>
  .toolbar {
    height: var(--toolbar-height);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border);
  }

  .title {
    flex: 1;
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    text-align: center;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: background var(--transition);
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-btn.small {
    width: 24px;
    height: 24px;
  }

  .zoom-group {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius);
    background: var(--color-bg);
  }

  .zoom-value {
    min-width: 38px;
    padding: 2px 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
    border-radius: var(--radius);
  }

  .zoom-value:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .window-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: 4px;
    padding-left: 10px;
    border-left: 1px solid var(--color-border);
  }

  .close-btn:hover {
    background: #e81123;
    color: #fff;
  }
</style>
