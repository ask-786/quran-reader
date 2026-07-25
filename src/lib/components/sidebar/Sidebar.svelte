<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { resolve } from '$app/paths';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  let filter = $state('');

  onMount(() => {
    surahsStore.init();
  });

  const activeId = $derived(Number($page.params.id ?? 0));

  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return surahsStore.list;
    return surahsStore.list.filter(
      (s) =>
        s.name_ar.includes(filter.trim()) ||
        s.transliteration.toLowerCase().includes(q) ||
        String(s.id) === q,
    );
  });
</script>

<aside class="sidebar scrollbar-thin">
  <div class="filter-row">
    <input
      class="filter-input"
      type="text"
      placeholder="Find a surah…"
      bind:value={filter}
      aria-label="Filter surahs"
    />
  </div>

  {#if surahsStore.loading}
    <p class="hint">Loading surahs…</p>
  {:else if surahsStore.error}
    <p class="hint hint-error">Failed to load surahs.</p>
  {:else}
    <ul class="surah-list">
      {#each filtered as surah (surah.id)}
        <li>
          <a
            class="surah-item"
            class:active={surah.id === activeId}
            href={resolve('/surah/[id]', { id: String(surah.id) })}
          >
            <span class="surah-number">{surah.id}</span>
            <span class="surah-info">
              <span class="surah-translit">{surah.transliteration}</span>
              <span class="surah-meta">{surah.revelation_type} · {surah.verses_count} verses</span>
            </span>
            <span class="surah-arabic quran-text">{surah.name_ar}</span>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    flex-shrink: 0;
    height: 100%;
    overflow-y: auto;
    background: var(--color-bg-elevated);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
  }

  .filter-row {
    padding: 12px;
    position: sticky;
    top: 0;
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border);
    z-index: 1;
  }

  .filter-input {
    width: 100%;
    padding: 8px 12px;
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

  .surah-list {
    list-style: none;
    margin: 0;
    padding: 4px;
  }

  .surah-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: var(--radius);
    text-decoration: none;
    color: var(--color-text);
    transition: background var(--transition);
  }

  .surah-item:hover {
    background: var(--color-bg-hover);
  }

  .surah-item.active {
    background: var(--color-bg-hover);
    box-shadow: inset 2px 0 0 var(--color-accent);
  }

  .surah-number {
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

  .surah-arabic {
    font-size: 17px;
    line-height: 1;
    flex-shrink: 0;
    color: var(--color-text);
  }
</style>
