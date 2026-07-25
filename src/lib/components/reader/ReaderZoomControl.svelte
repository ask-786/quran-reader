<script lang="ts">
  import { Minus, Plus, ALargeSmall } from 'lucide-svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
</script>

<div class="zoom-group" title="Reader text zoom">
  <ALargeSmall size={15} class="zoom-group-icon" />
  <button
    class="icon-btn small"
    onclick={() => settingsStore.zoomReaderOut()}
    aria-label="Decrease reader zoom"
  >
    <Minus size={13} />
  </button>
  <button
    class="zoom-value"
    onclick={() => settingsStore.resetReaderZoom()}
    aria-label="Reset reader zoom"
  >
    {Math.round(settingsStore.current.reader_zoom * 100)}%
  </button>
  <button
    class="icon-btn small"
    onclick={() => settingsStore.zoomReaderIn()}
    aria-label="Increase reader zoom"
  >
    <Plus size={13} />
  </button>
</div>

<style>
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

  .zoom-group :global(.zoom-group-icon) {
    color: var(--color-text-muted);
    margin: 0 2px 0 4px;
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
</style>
