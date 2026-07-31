<script lang="ts">
  import { onMount, tick } from 'svelte';
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
  let inputEl = $state<HTMLInputElement>();
  // The keyboard cursor over the result list. -1 is "not navigating yet": the
  // list stays unmarked while you type, and only an arrow key picks a row.
  let activeIndex = $state(-1);
  let inputFocused = $state(false);

  onMount(() => {
    surahsStore.init();
  });

  $effect(() => {
    if (!uiStore.searchFocusPending) return;
    uiStore.searchFocusPending = false;
    // A tick, because the shortcut may have just opened a collapsed sidebar or
    // left focus mode, and this input isn't in the DOM until that lands.
    tick().then(() => {
      inputEl?.focus();
      inputEl?.select();
    });
  });

  // Any change to what's listed retires the cursor — the row it pointed at is
  // gone or has moved, and re-typing means you're choosing again, not yet
  // navigating.
  $effect(() => {
    filter;
    mode;
    activeIndex = -1;
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

  // How many rows the cursor has to work with, whichever mode is showing.
  const resultCount = $derived(
    mode === 'surah'
      ? filteredSurahs.length
      : mode === 'juz'
        ? filteredJuz.length
        : filteredHizb.length,
  );

  const optionId = (i: number) => `sidebar-option-${i}`;

  function moveCursor(delta: number) {
    if (resultCount === 0) return;
    if (activeIndex < 0) {
      // Entering the list from the input: down lands on the first result, up on
      // the last.
      activeIndex = delta > 0 ? 0 : resultCount - 1;
    } else {
      // Clamped, not wrapped: with 114 surahs, falling off the top and landing
      // at the bottom loses your place more than it helps.
      activeIndex = Math.min(Math.max(activeIndex + delta, 0), resultCount - 1);
    }
    tick().then(() =>
      document.getElementById(optionId(activeIndex))?.scrollIntoView({ block: 'nearest' }),
    );
  }

  function onInputKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        moveCursor(1);
        return;
      case 'ArrowUp':
        e.preventDefault();
        moveCursor(-1);
        return;
      case 'Enter': {
        // Nothing is picked until an arrow key picks it, so a bare Enter after
        // typing does nothing rather than opening a result you never looked at.
        if (activeIndex < 0) return;
        const option = document.getElementById(optionId(activeIndex));
        if (!option) return;
        e.preventDefault();
        // Hand the keyboard back to the reader — the point of choosing a result
        // is to go read it, and the reader's own keys are all bare letters that
        // a focused input would swallow.
        inputEl?.blur();
        // Clicking the link itself rather than routing by hand, so the keyboard
        // and the mouse go through one code path.
        option.click();
        return;
      }
      case 'Escape':
        // First press clears a filter, second leaves the box.
        if (filter) filter = '';
        else inputEl?.blur();
        return;
    }
  }
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
        bind:this={inputEl}
        bind:value={filter}
        onkeydown={onInputKeydown}
        onfocus={() => (inputFocused = true)}
        onblur={() => {
          inputFocused = false;
          // Don't leave a cursor parked out of sight, ready to fire on the next
          // Enter — coming back to the box starts from "not navigating" again.
          activeIndex = -1;
        }}
        aria-label={placeholder}
        autocomplete="off"
        role="combobox"
        aria-expanded="true"
        aria-controls="sidebar-results"
        aria-autocomplete="list"
        aria-activedescendant={inputFocused && activeIndex >= 0 ? optionId(activeIndex) : undefined}
      />
    </div>
  </div>

  {#if mode === 'surah'}
    {#if surahsStore.loading}
      <p class="hint">Loading surahs…</p>
    {:else if surahsStore.error}
      <p class="hint hint-error">Failed to load surahs.</p>
    {:else}
      <ul class="item-list" id="sidebar-results" role="listbox">
        {#each filteredSurahs as surah, i (surah.id)}
          <li>
            <a
              class="surah-item"
              class:active={surah.id === activeSurahId}
              class:cursor={inputFocused && i === activeIndex}
              id={optionId(i)}
              role="option"
              aria-selected={inputFocused && i === activeIndex}
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
    <ul class="item-list" id="sidebar-results" role="listbox">
      {#each filteredJuz as n, i (n)}
        <li>
          <a
            class="unit-item"
            class:active={n === activeJuz}
            class:cursor={inputFocused && i === activeIndex}
            id={optionId(i)}
            role="option"
            aria-selected={inputFocused && i === activeIndex}
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
    <ul class="item-list" id="sidebar-results" role="listbox">
      {#each filteredHizb as n, i (n)}
        <li>
          <a
            class="unit-item"
            class:active={n === activeHizb}
            class:cursor={inputFocused && i === activeIndex}
            id={optionId(i)}
            role="option"
            aria-selected={inputFocused && i === activeIndex}
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
    /* Clears the sticky header when the keyboard cursor scrolls an item in. */
    scroll-margin: 92px 0 8px;
  }

  .surah-item:hover,
  .unit-item:hover {
    background: var(--color-bg-hover);
  }

  .surah-item.active,
  .unit-item.active {
    background: var(--color-bg-hover);
  }

  /* Outlined rather than filled, so it stays legible on the .active item —
     where you are and what you're about to pick are different things. */
  .surah-item.cursor,
  .unit-item.cursor {
    background: var(--color-bg-hover);
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
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
