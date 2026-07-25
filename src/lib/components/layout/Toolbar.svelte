<script lang="ts">
  import { page } from '$app/stores';
  import { PanelLeft, Moon, Sun, Coffee, BookOpen, Rows3 } from 'lucide-svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Theme } from '$lib/types/database';

  const THEMES: Theme[] = ['dark', 'light', 'sepia'];

  const surah = $derived(
    $page.data?.surah as { name_en: string; transliteration: string } | undefined,
  );

  function cycleTheme() {
    const i = THEMES.indexOf(settingsStore.current.theme);
    settingsStore.setTheme(THEMES[(i + 1) % THEMES.length]);
  }
</script>

<header class="toolbar">
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
</style>
