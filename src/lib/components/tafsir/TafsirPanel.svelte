<script lang="ts">
  import { X, PanelBottom } from 'lucide-svelte';
  import { tafsirStore, clampTafsirWidth } from '$lib/stores/tafsir.svelte';
  import TafsirMeta from './TafsirMeta.svelte';
  import TafsirBody from './TafsirBody.svelte';

  /** Live width while dragging; null when the stored width is in force. */
  let dragWidth = $state<number | null>(null);
  let dragging = $state(false);

  const width = $derived(dragWidth ?? tafsirStore.width);

  function startResize(e: PointerEvent) {
    const handle = e.currentTarget as HTMLElement;
    handle.setPointerCapture(e.pointerId);
    dragging = true;
    dragWidth = width;
  }

  function resize(e: PointerEvent) {
    if (!dragging) return;
    // The panel is pinned to the right edge, so its width is whatever is left
    // between the pointer and that edge.
    dragWidth = clampTafsirWidth(window.innerWidth - e.clientX);
  }

  function endResize(e: PointerEvent) {
    if (!dragging) return;
    const handle = e.currentTarget as HTMLElement;
    handle.releasePointerCapture(e.pointerId);
    dragging = false;
    if (dragWidth !== null) void tafsirStore.setWidth(dragWidth);
    dragWidth = null;
  }

  function nudge(e: KeyboardEvent) {
    const step = e.shiftKey ? 40 : 10;
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      void tafsirStore.setWidth(width + step);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      void tafsirStore.setWidth(width - step);
    }
  }
</script>

<aside class="tafsir-panel" style:width="{width}px" aria-label="Tafsir">
  <!-- A button, not a div with role="separator": the ARIA splitter pattern
       wants a focusable widget, and a real button is the one element that is
       focusable, keyboard-operable and announced as interactive without any
       of it having to be re-declared. A button's role doesn't carry
       aria-valuenow, so the current width goes in the label instead — the
       arrow keys change it, so it has to be announced somewhere. -->
  <button
    type="button"
    class="resize-handle"
    class:dragging
    aria-label="Resize tafsir panel, currently {width} pixels wide"
    onpointerdown={startResize}
    onpointermove={resize}
    onpointerup={endResize}
    onpointercancel={endResize}
    onkeydown={nudge}
  ></button>

  <header class="panel-header">
    <TafsirMeta />
    <div class="actions">
      <button
        class="icon-btn"
        onclick={() => tafsirStore.setView('popover')}
        aria-label="Show tafsir as a popover instead"
        title="Show as popover"
      >
        <PanelBottom size={16} />
      </button>
      <button
        class="icon-btn"
        onclick={() => tafsirStore.setPanelOpen(false)}
        aria-label="Close tafsir"
      >
        <X size={16} />
      </button>
    </div>
  </header>

  <div class="panel-body">
    <TafsirBody />
  </div>
</aside>

<style>
  .tafsir-panel {
    position: relative;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border);
    z-index: 7;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    left: -3px;
    width: 7px;
    height: 100%;
    padding: 0;
    border: none;
    cursor: col-resize;
    background: transparent;
    z-index: 1;
  }

  .resize-handle:hover,
  .resize-handle:focus-visible,
  .resize-handle.dragging {
    background: var(--color-accent);
    opacity: 0.35;
    outline: none;
  }

  .panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 12px 10px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  /* Meta and body styles now live with TafsirMeta/TafsirBody, which the
     popover shares — keeping copies here is how the two surfaces drift. */
  .actions {
    display: flex;
    flex-shrink: 0;
    gap: 2px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .panel-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 16px 32px;
  }

  /* Below the tablet breakpoint the drawer covers the reader rather than
     squeezing it into a column too narrow to read. */
  @media (max-width: 900px) {
    .tafsir-panel {
      position: absolute;
      inset: 0 0 0 auto;
      width: min(100%, 480px) !important;
      box-shadow: -4px 0 24px rgba(0, 0, 0, 0.3);
    }

    .resize-handle {
      display: none;
    }
  }
</style>
