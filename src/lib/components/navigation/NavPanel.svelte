<!--
  The navigation panel: mode tabs, a filter box, and the result list. Rendered
  twice — docked in the sidebar and inside the command palette — off the one
  `Navigator` so both surfaces list the same things, filter them the same way,
  and answer to the same keys.

  Layout is a fixed head over a scrolling list in both variants, which is what
  keeps the palette's head still: switching tabs changes the list and nothing
  above it. Every row of the head is present in every mode for the same reason,
  including the hint line, which reserves its space whether or not it has
  something to say.
-->
<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { get } from 'svelte/store';
  import { page } from '$app/stores';
  import { CornerDownLeft, Search } from 'lucide-svelte';
  import { NAV_MODES, Navigator, type NavMode } from '$lib/navigation/navigator.svelte';
  import { readingStore } from '$lib/stores/reading.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { stripTashkeel } from '$lib/utils/arabic-text';

  type Variant = 'sidebar' | 'palette';

  let {
    variant = 'sidebar',
    onselect,
    onclose,
  }: {
    variant?: Variant;
    /** A row was opened. The sidebar closes its drawer; the palette closes. */
    onselect?: () => void;
    /** Escape on an empty filter — as far as this panel is concerned, "done". */
    onclose?: () => void;
  } = $props();

  const isPalette = $derived(variant === 'palette');
  // Read once and deliberately: which surface this is decides how the navigator
  // behaves for its whole life, and no caller swaps a panel's variant under it.
  const nav = new Navigator({ autoSelect: untrack(() => variant) === 'palette' });

  let inputEl = $state<HTMLInputElement>();
  let listEl = $state<HTMLElement>();
  let inputFocused = $state(false);

  // Seeded before the first render rather than from an effect: the palette
  // decides which tab to open on from the route, and it has to have decided
  // by the time it is painted.
  {
    const current = get(page);
    nav.setRoute(current.route.id ?? null, Number(current.params.id) || 0);
    if (nav.autoSelect) nav.syncToRoute();
  }

  const routeId = $derived($page.route.id ?? null);
  const routeParam = $derived(Number($page.params.id) || 0);
  $effect(() => {
    nav.setRoute(routeId, routeParam);
  });

  // The ring only shows where the keys it belongs to are live. In the palette
  // the box always has focus, so it always does.
  const showCursor = $derived(isPalette || inputFocused);

  const placeholder = $derived(
    nav.mode === 'surah'
      ? 'Find a surah, or 2:255 · p255'
      : nav.mode === 'juz'
        ? 'Jump to a Juz, or 2:255 · p255'
        : nav.mode === 'hizb'
          ? 'Jump to a Hizb, or 2:255 · p255'
          : 'Filter recent reading…',
  );

  const hint = $derived(
    nav.error ||
      (nav.inSurah
        ? 'Ayah in this surah, Surah:Ayah, or p + page number'
        : 'Surah:Ayah (2:255), or p + page number (p255)'),
  );

  onMount(() => {
    surahsStore.init();
    if (nav.mode === 'recent') void readingStore.refreshHistory();
    if (isPalette) {
      inputEl?.focus();
      scrollCursorIntoView();
    }
  });

  // Pull-based, not pushed on every write: the list only has to be right while
  // it is on screen, and reloading it as the reader scrolls would re-render
  // this panel throughout a reading session for nobody's benefit.
  $effect(() => {
    if (nav.mode !== 'recent') return;
    void readingStore.refreshHistory();
  });

  function scrollCursorIntoView() {
    const i = nav.activeIndex;
    if (i < 0) return;
    void tick().then(() =>
      document.getElementById(nav.optionId(i))?.scrollIntoView({ block: 'nearest' }),
    );
  }

  function move(delta: number) {
    nav.move(delta);
    scrollCursorIntoView();
  }

  function selectMode(mode: NavMode) {
    nav.setMode(mode);
    // Changing tabs from the tab strip is still choosing a destination, so the
    // keys go back to the box rather than being stranded on a button.
    if (isPalette) inputEl?.focus();
    if (listEl) listEl.scrollTop = 0;
  }

  function cycleMode(delta: number) {
    nav.cycleMode(delta);
    if (listEl) listEl.scrollTop = 0;
  }

  /**
   * Open the row Enter is pointing at by clicking its own anchor, so the
   * keyboard and the mouse go through exactly one code path.
   */
  function activate() {
    const i = nav.targetIndex;
    if (i < 0) return false;
    const option = document.getElementById(nav.optionId(i));
    if (!option) return false;
    // Hand the keyboard back to the reader — the point of choosing a result is
    // to go read it, and the reader's own keys are bare letters that a focused
    // input would swallow.
    inputEl?.blur();
    option.click();
    return true;
  }

  function onInputKeydown(e: KeyboardEvent) {
    // Alt/Ctrl + arrows step the tab strip from inside the box on both
    // surfaces; the palette also takes bare Tab, which it can afford because
    // nothing else in it wants the key.
    if ((e.altKey || e.ctrlKey || e.metaKey) && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
      e.preventDefault();
      cycleMode(e.key === 'ArrowRight' ? 1 : -1);
      return;
    }
    if (e.altKey || e.ctrlKey || e.metaKey) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        e.stopPropagation();
        move(1);
        return;
      case 'ArrowUp':
        e.preventDefault();
        e.stopPropagation();
        move(-1);
        return;
      case 'PageDown':
        e.preventDefault();
        e.stopPropagation();
        move(8);
        return;
      case 'PageUp':
        e.preventDefault();
        e.stopPropagation();
        move(-8);
        return;
      case 'Tab':
        if (!isPalette) return;
        e.preventDefault();
        cycleMode(e.shiftKey ? -1 : 1);
        return;
      case 'Enter':
        // In the sidebar nothing is picked until an arrow key picks it, so a
        // bare Enter after typing does nothing rather than opening a result you
        // never looked at — unless what you typed was a `2:255`, which is an
        // instruction, not a filter.
        if (!activate()) return;
        e.preventDefault();
        e.stopPropagation();
        return;
      case 'Escape':
        // First press clears the filter, second is "done with this panel".
        e.stopPropagation();
        if (nav.query) {
          nav.setQuery('');
          return;
        }
        if (onclose) onclose();
        else inputEl?.blur();
        return;
    }
  }
</script>

<!-- Every href in the list is built by the navigator, which resolves each route
     itself; the rule only recognises a resolve() written inline in the
     attribute, so it can't see that and is turned off across the list. -->
<!-- eslint-disable svelte/no-navigation-without-resolve -->
<div class="nav-panel" class:palette={isPalette}>
  <div class="nav-head">
    <div class="mode-tabs" role="tablist" aria-label="Browse by">
      {#each NAV_MODES as m (m.id)}
        <button
          class="mode-tab"
          class:active={nav.mode === m.id}
          role="tab"
          aria-selected={nav.mode === m.id}
          tabindex={isPalette ? -1 : 0}
          onclick={() => selectMode(m.id)}
        >
          {m.label}
        </button>
      {/each}
    </div>

    <div class="filter-row">
      <span class="filter-icon" aria-hidden="true"><Search size={15} /></span>
      <input
        class="filter-input"
        type="text"
        {placeholder}
        bind:this={inputEl}
        value={nav.query}
        oninput={(e) => nav.setQuery(e.currentTarget.value)}
        onkeydown={onInputKeydown}
        onfocus={() => (inputFocused = true)}
        onblur={() => {
          inputFocused = false;
          // Don't leave a cursor parked out of sight in the sidebar, ready to
          // fire on the next Enter. The palette's box never really loses focus.
          if (!isPalette) nav.resetCursor();
        }}
        aria-label={placeholder}
        autocomplete="off"
        role="combobox"
        aria-expanded="true"
        aria-controls="{nav.panelId}-results"
        aria-autocomplete="list"
        aria-activedescendant={showCursor ? nav.activeOptionId : undefined}
      />
    </div>

    <p class="hint" class:error={!!nav.error}>{hint}</p>
  </div>

  <div class="nav-body scrollbar-thin" bind:this={listEl}>
    {#if nav.loading}
      <p class="status">Loading…</p>
    {:else if nav.loadError}
      <p class="status status-error">Failed to load surahs.</p>
    {:else if nav.entries.length === 0}
      <p class="status">
        {#if nav.mode === 'recent' && !nav.query}
          Nothing read yet. Wherever you stop, this is where you'll find your way back.
        {:else}
          No match for “{nav.query}”.
        {/if}
      </p>
    {:else}
      <ul class="item-list" id="{nav.panelId}-results" role="listbox">
        {#each nav.entries as entry, i (entry.key)}
          {@const marked = showCursor && i === nav.activeIndex}
          <li>
            <a
              class="row row-{entry.kind}"
              class:active={entry.kind !== 'jump' && entry.kind !== 'recent' && entry.active}
              class:cursor={marked}
              id={nav.optionId(i)}
              role="option"
              aria-selected={marked}
              href={entry.href}
              onmousemove={() => nav.moveTo(i)}
              onclick={() => onselect?.()}
            >
              {#if entry.kind === 'jump'}
                <span class="roundel roundel-jump"><CornerDownLeft size={13} /></span>
                <span class="row-main">
                  <span class="row-title">Go to {entry.label}</span>
                  <span class="row-meta">{entry.detail}</span>
                </span>
              {:else if entry.kind === 'surah'}
                <span class="roundel">{entry.surah.id}</span>
                <span class="row-main">
                  <span class="row-title">{entry.surah.transliteration}</span>
                  <span class="row-meta"
                    >{entry.surah.revelation_type} · {entry.surah.verses_count} verses</span
                  >
                </span>
                <span class="surah-arabic">{stripTashkeel(entry.surah.name_ar)}</span>
              {:else if entry.kind === 'unit'}
                <span class="roundel">{entry.n}</span>
                <span class="row-main">
                  <span class="row-title">{entry.unit === 'juz' ? 'Juz' : 'Hizb'} {entry.n}</span>
                </span>
              {:else}
                <span class="row-main">
                  <span class="recent-head">
                    <span class="row-title">{entry.scope}</span>
                    <span class="recent-when">{entry.when}</span>
                  </span>
                  <span class="row-meta recent-range">
                    {entry.range}
                    {#if entry.page !== null}<span class="recent-page">· page {entry.page}</span
                      >{/if}
                  </span>
                </span>
              {/if}
            </a>
          </li>
        {/each}
      </ul>
    {/if}

    {#if !isPalette && nav.mode === 'recent' && readingStore.history.length > 0}
      <button class="clear-history" onclick={() => readingStore.clearHistory()}>
        Clear history
      </button>
    {/if}
  </div>
</div>

<!-- eslint-enable svelte/no-navigation-without-resolve -->

<style>
  .nav-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--color-bg-elevated);
  }

  /* Fixed by construction, not by a magic number: three rows that are always
     present, so the head is the same height in every mode and only the list
     below it changes. */
  .nav-head {
    flex: 0 0 auto;
    border-bottom: 1px solid var(--color-border);
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
    position: relative;
    display: flex;
    align-items: center;
    padding: 10px 12px 0;
  }

  .filter-icon {
    position: absolute;
    left: 22px;
    display: flex;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .filter-input {
    width: 100%;
    padding: 8px 10px 8px 32px;
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
    margin: 0;
    padding: 6px 12px 10px;
    font-size: 11px;
    line-height: 1.3;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hint.error {
    color: #d9736a;
  }

  .nav-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .status {
    padding: 16px;
    margin: 0;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .status-error {
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

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: var(--radius);
    text-decoration: none;
    color: var(--color-text);
    transition: background var(--transition);
    scroll-margin: 6px 0;
  }

  .row:hover {
    background: var(--color-bg-hover);
  }

  .row.active {
    background: var(--color-bg-hover);
  }

  /* Outlined rather than filled, so it stays legible on the .active row —
     where you are and what you're about to pick are different things. */
  .row.cursor {
    background: var(--color-bg-hover);
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .row.active .row-title,
  .row.active .roundel {
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

  .roundel-jump {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .row-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .row-title {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-meta {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  /* Live-shaped, so it needs --font-surah-name rather than the --font-quran
     the reader uses. See app.css for why the two are separate. */
  .surah-arabic {
    font-family: var(--font-surah-name);
    font-size: 17px;
    line-height: 1;
    flex-shrink: 0;
    color: var(--color-text);
  }

  /* A sitting has to say both what was read and when, and neither fits on the
     end of the other at sidebar width — so its two lines are a header row and
     a range, rather than the single row the other tabs use. */
  .recent-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .recent-when {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .recent-range {
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .recent-page {
    opacity: 0.75;
  }

  .clear-history {
    align-self: flex-start;
    margin: 2px 12px 16px;
    padding: 6px 10px;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: background var(--transition);
  }

  .clear-history:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  /* The palette is wider and sits under the eye rather than off to the side,
     so it can afford a little more room per row. */
  .palette .mode-tabs,
  .palette .filter-row {
    padding-left: 14px;
    padding-right: 14px;
  }

  .palette .filter-icon {
    left: 26px;
  }

  .palette .filter-input {
    padding: 10px 12px 10px 34px;
    font-size: 15px;
  }

  .palette .hint {
    padding-left: 14px;
    padding-right: 14px;
  }

  .palette .item-list {
    padding: 6px;
  }
</style>
