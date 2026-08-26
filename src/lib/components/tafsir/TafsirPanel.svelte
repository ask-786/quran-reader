<script lang="ts">
  import { X, PanelBottom, Library } from 'lucide-svelte';
  import { tafsirStore, clampTafsirWidth } from '$lib/stores/tafsir.svelte';
  import TafsirMeta from './TafsirMeta.svelte';
  import TafsirBody from './TafsirBody.svelte';
  import TafsirEditions from './TafsirEditions.svelte';

  let dragging = $state(false);

  // The live width lives in the store, not here: the reader's right-edge
  // controls offset by it to stay out from under the panel, and they have to
  // follow the drag frame by frame. See `liveWidth` there.
  const width = $derived(tafsirStore.liveWidth);

  function startResize(e: PointerEvent) {
    const handle = e.currentTarget as HTMLElement;
    handle.setPointerCapture(e.pointerId);
    dragging = true;
    tafsirStore.dragWidth = width;
  }

  function resize(e: PointerEvent) {
    if (!dragging) return;
    // The panel is pinned to the right edge, so its width is whatever is left
    // between the pointer and that edge.
    tafsirStore.dragWidth = clampTafsirWidth(window.innerWidth - e.clientX);
  }

  function endResize(e: PointerEvent) {
    if (!dragging) return;
    const handle = e.currentTarget as HTMLElement;
    handle.releasePointerCapture(e.pointerId);
    dragging = false;
    const dragged = tafsirStore.dragWidth;
    if (dragged !== null) void tafsirStore.setWidth(dragged);
    tafsirStore.dragWidth = null;
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

<aside class="tafsir-panel" aria-label="Tafsir">
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
        class:on={tafsirStore.managing}
        onclick={() => (tafsirStore.managing = !tafsirStore.managing)}
        aria-pressed={tafsirStore.managing}
        aria-label="Manage tafsir editions"
        title="Editions"
      >
        <Library size={16} />
      </button>
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
    {#if tafsirStore.managing}
      <TafsirEditions />
    {:else}
      <TafsirBody />
    {/if}
  </div>
</aside>

<style>
  /* An overlay at every width, not a column beside the reader: the measure of
     the reading text is a typographic decision the reader made with the zoom
     control, and opening a reference panel is not a reason to re-flow every
     line of it. The page underneath keeps its width; this covers part of it.

     Width comes from --tafsir-inset rather than an inline style so that one
     value drives the panel *and* the reader controls that step aside for it —
     including under the breakpoint below, where the panel's width stops
     tracking the stored one. ReaderPage defines it. */
  .tafsir-panel {
    position: absolute;
    inset: 0 0 0 auto;
    width: var(--tafsir-inset);
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border);
    /* Above ReaderContextBar's 7: both are chrome over the reader, and a
       floating bar showing through a solid panel reads as a bug. */
    z-index: 8;
    box-shadow: -4px 0 24px rgb(0 0 0 / 0.28);
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

  /* The editions view is a mode the panel stays in, not a one-shot action, so
     its button holds a pressed state rather than only reacting to hover. */
  .icon-btn.on {
    background: var(--color-bg-hover);
    color: var(--color-accent);
  }

  .panel-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 16px 32px;
  }

  /* Below the tablet breakpoint there is no room left to give: the panel takes
     the window and the drag handle has nothing useful to do. The width itself
     is capped in ReaderPage, with --tafsir-inset, so the controls that step
     aside for the panel agree with it here too. */
  @media (max-width: 900px) {
    .resize-handle {
      display: none;
    }
  }
</style>
