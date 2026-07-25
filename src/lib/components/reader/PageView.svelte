<script lang="ts">
  import { untrack } from 'svelte';
  import { ChevronRight, ChevronLeft } from 'lucide-svelte';
  import { getPage } from '$lib/api/db';
  import type { MushafPage } from '$lib/types/database';
  import { loadPageFont, loadBasmalaFont } from '$lib/utils/mushaf-fonts';

  let {
    page = 1,
    onPageChange,
  }: {
    page?: number;
    onPageChange?: (nextPage: number) => void;
  } = $props();

  let data = $state<MushafPage | null>(null);
  let fontFamily = $state<string | null>(null);
  let basmalaFontFamily = $state<string | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let pageInput = $state(untrack(() => String(page)));

  $effect(() => {
    const p = page;
    pageInput = String(p);
    loading = true;
    error = null;

    let cancelled = false;

    (async () => {
      try {
        const [pageData, family, basmalaFamily] = await Promise.all([
          getPage(p),
          loadPageFont(p),
          loadBasmalaFont(),
        ]);
        if (cancelled) return;
        data = pageData;
        fontFamily = family;
        basmalaFontFamily = basmalaFamily;
      } catch (err) {
        if (!cancelled) error = err instanceof Error ? err.message : String(err);
      } finally {
        if (!cancelled) loading = false;
      }
    })();

    // Warm the neighbours so flipping forward/back doesn't stall on a font load.
    if (p < 604) void loadPageFont(p + 1);
    if (p > 1) void loadPageFont(p - 1);

    return () => {
      cancelled = true;
    };
  });

  function goTo(target: number) {
    const clamped = Math.min(604, Math.max(1, target));
    if (clamped !== page) onPageChange?.(clamped);
  }

  function submitPageInput() {
    const n = Number(pageInput);
    if (Number.isInteger(n)) goTo(n);
    else pageInput = String(page);
  }
</script>

<div class="page-view">
  <div class="page-nav">
    <button
      class="nav-btn"
      onclick={() => goTo(page + 1)}
      disabled={page >= 604}
      aria-label="Next page"
    >
      <ChevronLeft size={18} />
    </button>

    <div class="page-indicator">
      <span>Page</span>
      <input
        type="text"
        inputmode="numeric"
        bind:value={pageInput}
        onchange={submitPageInput}
        aria-label="Go to page"
      />
      <span>/ 604</span>
    </div>

    <button
      class="nav-btn"
      onclick={() => goTo(page - 1)}
      disabled={page <= 1}
      aria-label="Previous page"
    >
      <ChevronRight size={18} />
    </button>
  </div>

  <div class="page-surface scrollbar-thin">
    {#if error}
      <p class="state-message">Couldn't load page {page}: {error}</p>
    {:else if loading || !data}
      <p class="state-message">Loading…</p>
    {:else}
      <div class="mushaf-page" dir="rtl">
        {#each data.lines as line (line.line_number)}
          {#if line.line_type === 'surah_header'}
            <div class="line surah-header-line">
              <span class="motif" aria-hidden="true">۞</span>
              <h3 class="quran-text">{line.text}</h3>
              <span class="motif" aria-hidden="true">۞</span>
            </div>
          {:else if line.line_type === 'basmala'}
            <div class="line basmala-line" style:font-family={basmalaFontFamily}>
              {#each line.words as w (w.position)}
                <span aria-label={w.uthmani_text}>{w.glyph_v2}</span>
              {/each}
            </div>
          {:else}
            <div class="line text-line" style:font-family={fontFamily}>
              {#each line.words as w (w.position)}
                <span
                  class="word"
                  data-ayah-id={w.ayah_id}
                  aria-label={w.uthmani_text}
                  title={w.uthmani_text}>{w.glyph_v2}</span
                >
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .page-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .page-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 10px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: var(--transition);
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .nav-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .page-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .page-indicator input {
    width: 44px;
    padding: 3px 4px;
    text-align: center;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text);
  }

  .page-surface {
    flex: 1;
    overflow-y: auto;
    display: flex;
    justify-content: center;
    padding: 24px 16px 40px;
  }

  .mushaf-page {
    width: min(560px, 100%);
  }

  .line {
    display: flex;
    align-items: baseline;
    min-height: 2.6em;
  }

  .text-line,
  .basmala-line {
    justify-content: space-between;
    flex-wrap: nowrap;
    font-size: 27px;
    line-height: 2.5;
    color: var(--color-text);
  }

  .basmala-line {
    justify-content: center;
    gap: 0.3em;
    color: var(--color-accent);
  }

  .word {
    cursor: default;
  }

  .word:hover {
    color: var(--color-accent);
  }

  .surah-header-line {
    justify-content: center;
    align-items: center;
    gap: 10px;
    margin: 10px 0;
    padding: 8px 0;
    border-top: 1px solid var(--color-accent);
    border-bottom: 1px solid var(--color-accent);
    color: var(--color-accent);
  }

  .surah-header-line h3 {
    margin: 0;
    font-size: 22px;
    white-space: nowrap;
  }

  .motif {
    font-size: 13px;
  }

  .state-message {
    color: var(--color-text-muted);
    font-size: 14px;
    text-align: center;
    margin-top: 40px;
  }
</style>
