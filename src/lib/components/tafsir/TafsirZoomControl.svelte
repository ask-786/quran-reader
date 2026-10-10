<script lang="ts">
  import { Plus, Minus } from 'lucide-svelte';
  import { tafsirStore, TAFSIR_ZOOM_MIN, TAFSIR_ZOOM_MAX } from '$lib/stores/tafsir.svelte';

  /**
   * Text size for the commentary, shared by the card and the side panel. The
   * same shape as the reader's zoom control — out, level, in, with the level
   * itself as the reset — laid on its side to sit in a header.
   *
   * `size` matches the icon size of the header it sits in, so the group lines
   * up with the buttons beside it.
   */
  let { size = 16 }: { size?: number } = $props();

  const zoom = $derived(tafsirStore.zoom);
</script>

<div class="zoom-group" style:--btn={`${size + 12}px`}>
  <button
    class="icon-btn"
    onclick={() => tafsirStore.zoomOut()}
    disabled={zoom <= TAFSIR_ZOOM_MIN}
    aria-label="Smaller commentary text"
    title="Smaller text"
  >
    <Minus size={size - 2} />
  </button>
  <button
    class="zoom-value"
    onclick={() => tafsirStore.resetZoom()}
    aria-label="Reset commentary text size, currently {Math.round(zoom * 100)}%"
    title="Reset text size"
  >
    {Math.round(zoom * 100)}%
  </button>
  <button
    class="icon-btn"
    onclick={() => tafsirStore.zoomIn()}
    disabled={zoom >= TAFSIR_ZOOM_MAX}
    aria-label="Larger commentary text"
    title="Larger text"
  >
    <Plus size={size - 2} />
  </button>
</div>

<style>
  .zoom-group {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    gap: 1px;
    /* Set off from the verse and surface buttons beside it: those move the
       card, this changes what is in it. */
    margin-inline-end: 4px;
    padding-inline-end: 4px;
    border-inline-end: 1px solid var(--color-border);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: calc(var(--btn) - 4px);
    height: var(--btn);
    padding: 0;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  /* Wide enough for "200%", so the group does not shift as the level
     changes. */
  .zoom-value {
    min-width: 34px;
    height: var(--btn);
    padding: 0 2px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    cursor: pointer;
  }

  .icon-btn:hover:not(:disabled),
  .zoom-value:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
