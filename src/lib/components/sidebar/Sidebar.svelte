<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { resolve } from '$app/paths';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { stripTashkeel } from '$lib/utils/arabic-text';
  import { isNarrowViewport } from '$lib/utils/viewport';

  type Mode = 'surah' | 'juz' | 'hizb';

  const MODES: { id: Mode; label: string }[] = [
    { id: 'surah', label: 'Surah' },
    { id: 'juz', label: 'Juz' },
    { id: 'hizb', label: 'Hizb' },
  ];

  const JUZ_LIST = Array.from({ length: 30 }, (_, i) => i + 1);
  const HIZB_LIST = Array.from({ length: 60 }, (_, i) => i + 1);

  let mode = $state<Mode>('surah');
  let filter = $state('');

  onMount(() => {
    surahsStore.init();
  });

  function selectMode(next: Mode) {
    mode = next;
    filter = '';
  }

  function selectItem() {
    if (isNarrowViewport()) uiStore.sidebarOpen = false;
  }

  const activeSurahId = $derived(
    $page.url.pathname.startsWith('/surah/') ? Number($page.params.id) : 0,
  );
  const activeJuz = $derived($page.url.pathname.startsWith('/juz/') ? Number($page.params.id) : 0);
  const activeHizb = $derived(
    $page.url.pathname.startsWith('/hizb/') ? Number($page.params.id) : 0,
  );

  const filteredSurahs = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return surahsStore.list;
    // Both sides unvocalised: nobody types the Mushaf's harakat into a filter box.
    const qArabic = stripTashkeel(filter.trim());
    return surahsStore.list.filter(
      (s) =>
        stripTashkeel(s.name_ar).includes(qArabic) ||
        s.transliteration.toLowerCase().includes(q) ||
        String(s.id) === q,
    );
  });

  const filteredJuz = $derived.by(() => {
    const q = filter.trim();
    return q ? JUZ_LIST.filter((n) => String(n).includes(q)) : JUZ_LIST;
  });

  const filteredHizb = $derived.by(() => {
    const q = filter.trim();
    return q ? HIZB_LIST.filter((n) => String(n).includes(q)) : HIZB_LIST;
  });

  const placeholder = $derived(
    mode === 'surah' ? 'Find a surah…' : mode === 'juz' ? 'Jump to a Juz…' : 'Jump to a Hizb…',
  );
</script>

<aside class="sidebar scrollbar-thin">
  <div class="sidebar-header">
    <div class="mode-tabs">
      {#each MODES as m (m.id)}
        <button class="mode-tab" class:active={mode === m.id} onclick={() => selectMode(m.id)}>
          {m.label}
        </button>
      {/each}
    </div>

    <div class="filter-row">
      <input
        class="filter-input"
        type="text"
        {placeholder}
        bind:value={filter}
        aria-label={placeholder}
      />
    </div>
  </div>

  {#if mode === 'surah'}
    {#if surahsStore.loading}
      <p class="hint">Loading surahs…</p>
    {:else if surahsStore.error}
      <p class="hint hint-error">Failed to load surahs.</p>
    {:else}
      <ul class="item-list">
        {#each filteredSurahs as surah (surah.id)}
          <li>
            <a
              class="surah-item"
              class:active={surah.id === activeSurahId}
              href={resolve('/surah/[id]', { id: String(surah.id) })}
              onclick={selectItem}
            >
              <span class="roundel">{surah.id}</span>
              <span class="surah-info">
                <span class="surah-translit">{surah.transliteration}</span>
                <span class="surah-meta">{surah.revelation_type} · {surah.verses_count} verses</span
                >
              </span>
              <span class="surah-arabic quran-text">{stripTashkeel(surah.name_ar)}</span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  {:else if mode === 'juz'}
    <ul class="item-list">
      {#each filteredJuz as n (n)}
        <li>
          <a
            class="unit-item"
            class:active={n === activeJuz}
            href={resolve('/juz/[id]', { id: String(n) })}
            onclick={selectItem}
          >
            <span class="roundel">{n}</span>
            <span class="unit-label">Juz {n}</span>
          </a>
        </li>
      {/each}
    </ul>
  {:else}
    <ul class="item-list">
      {#each filteredHizb as n (n)}
        <li>
          <a
            class="unit-item"
            class:active={n === activeHizb}
            href={resolve('/hizb/[id]', { id: String(n) })}
            onclick={selectItem}
          >
            <span class="roundel">{n}</span>
            <span class="unit-label">Hizb {n}</span>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .sidebar {
    width: 100%;
    flex-shrink: 0;
    height: 100%;
    overflow-y: auto;
    background: var(--color-bg-elevated);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    position: sticky;
    top: 0;
    background: var(--color-bg-elevated);
    z-index: 1;
  }

  .mode-tabs {
    display: flex;
    gap: 2px;
    padding: 10px 12px 0;
  }

  .mode-tab {
    flex: 1;
    padding: 6px 0;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition);
  }

  .mode-tab:hover {
    background: var(--color-bg-hover);
  }

  .mode-tab.active {
    background: var(--color-bg-hover);
    color: var(--color-accent);
  }

  .filter-row {
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .filter-input {
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 14px;
  }

  .filter-input:focus {
    outline: 2px solid var(--color-accent);
    outline-offset: -1px;
  }

  .hint {
    padding: 16px;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .hint-error {
    color: #d9736a;
  }

  .item-list {
    list-style: none;
    margin: 0;
    padding: 6px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .surah-item,
  .unit-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: var(--radius);
    text-decoration: none;
    color: var(--color-text);
    transition: background var(--transition);
  }

  .surah-item:hover,
  .unit-item:hover {
    background: var(--color-bg-hover);
  }

  .surah-item.active,
  .unit-item.active {
    background: var(--color-bg-hover);
  }

  .surah-item.active .surah-translit,
  .surah-item.active .roundel,
  .unit-item.active .unit-label,
  .unit-item.active .roundel {
    color: var(--color-accent);
  }

  .roundel {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .surah-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .surah-translit {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .surah-meta {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  /* Live-shaped, so it needs --font-surah-name rather than the --font-quran
     that .quran-text supplies. See app.css for why the two are separate. */
  .surah-arabic {
    font-family: var(--font-surah-name);
    font-size: 17px;
    line-height: 1;
    flex-shrink: 0;
    color: var(--color-text);
  }

  .unit-label {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
  }
</style>
