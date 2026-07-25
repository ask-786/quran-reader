<script lang="ts">
  import { Bookmark, BookmarkCheck, Copy, Check } from 'lucide-svelte';
  import type { Ayah } from '$lib/types/database';
  import { toArabicNumerals } from '$lib/utils/arabic-numerals';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';

  let {
    ayah,
    translation = null,
    showAyahNumbers = true,
  }: {
    ayah: Ayah;
    translation?: string | null;
    showAyahNumbers?: boolean;
  } = $props();

  const bookmarked = $derived(bookmarksStore.isBookmarked(ayah.id));
  let copied = $state(false);

  async function copyText() {
    await navigator.clipboard.writeText(ayah.uthmani_text);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }
</script>

<div class="ayah-row" id="ayah-{ayah.id}" data-page={ayah.page} data-juz={ayah.juz}>
  <div class="ayah-actions">
    <button
      class="action-btn"
      class:active={bookmarked}
      onclick={() => bookmarksStore.toggle(ayah.id)}
      aria-pressed={bookmarked}
      aria-label={bookmarked ? 'Remove bookmark' : 'Add bookmark'}
    >
      {#if bookmarked}<BookmarkCheck size={15} />{:else}<Bookmark size={15} />{/if}
    </button>
    <button class="action-btn" onclick={copyText} aria-label="Copy ayah text">
      {#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}
    </button>
  </div>

  <p class="ayah-text quran-text">
    {ayah.uthmani_text}
    {#if ayah.sajdah}
      <span class="sajdah-mark" title="Sajdah — prostration recommended">۩</span>
    {/if}
    {#if showAyahNumbers}
      <span class="ayah-marker">{toArabicNumerals(ayah.ayah_number)}</span>
    {/if}
  </p>

  {#if translation}
    <p class="ayah-translation">{translation}</p>
  {/if}
</div>

<style>
  .ayah-row {
    position: relative;
    padding: 10px 8px 14px;
    border-radius: var(--radius);
    transition: background var(--transition);
  }

  .ayah-row:hover,
  .ayah-row:focus-within {
    background: var(--color-bg-hover);
  }

  .ayah-actions {
    position: absolute;
    top: 6px;
    left: 8px;
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity var(--transition);
  }

  .ayah-row:hover .ayah-actions,
  .ayah-row:focus-within .ayah-actions {
    opacity: 1;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  .action-btn.active {
    color: var(--color-accent);
  }

  .ayah-text {
    margin: 0;
  }

  .ayah-marker {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 0.9em;
    height: 0.9em;
    margin: 0 0.2em;
    border: 1px solid var(--color-accent);
    border-radius: 50%;
    font-size: 0.5em;
    vertical-align: middle;
    color: var(--color-accent);
  }

  .sajdah-mark {
    color: var(--color-sajdah);
    font-size: 0.7em;
    margin-inline-start: 0.2em;
  }

  .ayah-translation {
    margin: 8px 4px 0;
    font-size: 15px;
    line-height: 1.7;
    color: var(--color-text-muted);
    direction: ltr;
    text-align: left;
  }
</style>
