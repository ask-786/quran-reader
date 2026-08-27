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
    Focus,
    Search,
    ScrollText,
    Settings,
  } from 'lucide-svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import type { Theme } from '$lib/types/database';

  const THEMES: Theme[] = ['dark', 'light', 'sepia'];
  const appWindow = getCurrentWindow();

  const heading = $derived($page.data?.title as string | undefined);
  const hasReader = $derived(Array.isArray($page.data?.ayahs));

  // One meaning in both views: tafsir mode, not whatever it has put on screen.
  // The card and the panel come and go under a mode that stays put, so lighting
  // this for the panel being open made it read as a panel switch — and left the
  // mode itself with no control at all in that view.
  const tafsirOn = $derived(tafsirStore.clickOpens);
  const tafsirLabel = $derived(
    tafsirOn
      ? 'Turn off tafsir mode'
      : tafsirStore.view === 'panel'
        ? 'Turn on tafsir mode — click a verse to open its commentary in the panel'
        : 'Turn on tafsir mode — click a verse for its commentary',
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

<header class="toolbar scrollbar-none" data-tauri-drag-region>
  <button class="icon-btn" onclick={() => uiStore.toggleSidebar()} aria-label="Toggle sidebar">
    <PanelLeft size={18} />
  </button>

  <button
    class="icon-btn"
    onclick={() => uiStore.openPalette()}
    aria-label="Search and go to"
    title="Search and go to (Ctrl+K)"
  >
    <Search size={18} />
  </button>

  <h1 class="title" data-tauri-drag-region>{heading ?? 'Quran Reader'}</h1>

  {#if hasReader}
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

    <button
      class="icon-btn"
      class:active={tafsirOn}
      onclick={() => tafsirStore.toggle()}
      aria-pressed={tafsirOn}
      aria-label={tafsirLabel}
      title="{tafsirLabel} (t)"
    >
      <ScrollText size={18} />
    </button>
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

  <button
    class="icon-btn"
    class:active={uiStore.settingsOpen}
    onclick={() => uiStore.toggleSettings()}
    aria-pressed={uiStore.settingsOpen}
    aria-label="Settings"
    title="Settings (Ctrl+,)"
  >
    <Settings size={18} />
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
    overflow-x: auto;
  }

  .title {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .icon-btn,
  .window-controls {
    flex-shrink: 0;
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

  /* Toggles, unlike the one-shot buttons around them, have to show that they
     are currently on. */
  .icon-btn.active {
    color: var(--color-accent);
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

  @media (max-width: 480px) {
    .toolbar {
      gap: 4px;
      padding: 0 8px;
    }
  }
</style>
