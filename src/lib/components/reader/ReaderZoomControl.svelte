<script lang="ts">
  import { Plus, Minus } from 'lucide-svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let zoom = $derived(uiStore.readerZoom);
</script>

<div class="zoom-group" title={uiStore.focusMode ? 'Focus view zoom' : 'Reader text zoom'}>
  <button
    class="icon-btn"
    onclick={() => settingsStore.zoomReaderIn(uiStore.focusMode)}
    aria-label="Increase reader zoom"
  >
    <Plus size={14} />
  </button>
  <button
    class="zoom-value"
    onclick={() => settingsStore.resetReaderZoom(uiStore.focusMode)}
    aria-label="Reset reader zoom"
  >
    {Math.round(zoom * 100)}%
  </button>
  <button
    class="icon-btn"
    onclick={() => settingsStore.zoomReaderOut(uiStore.focusMode)}
    aria-label="Decrease reader zoom"
  >
    <Minus size={14} />
  </button>
</div>

<style>
  .zoom-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 4px;
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
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

  .zoom-value {
    padding: 2px 0;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
    border-radius: var(--radius);
  }

  .zoom-value:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }
</style>
