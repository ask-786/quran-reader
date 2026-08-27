<!--
  The command palette: everything the sidebar can do, over the reader, on the
  keyboard, without opening the sidebar or leaving focus mode.

  It is a fixed-size box on purpose. The head — tabs, filter, hint — is the same
  height in every mode, the list under it is a fixed-height scroller, and the
  key legend is pinned to the bottom, so tabbing from Surah (114 rows) to Juz
  (30) doesn't resize the dialog or move anything you were aiming at.
-->
<script lang="ts">
  import NavPanel from './NavPanel.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  // Which keys are live depends on whether the box has focus, so the legend
  // says different things in the two states rather than one thing that is half
  // wrong in each. Escape walks out of the palette a layer at a time — the
  // filter, then the box, then the palette — and the legend names the next
  // layer, which is the only one the key is about to do.
  let searchFocused = $state(true);

  const KEYS: { keys: string; label: string }[] = $derived(
    searchFocused
      ? [
          { keys: '↑↓', label: 'move' },
          { keys: '↵', label: 'open' },
          { keys: 'Tab', label: 'switch list' },
          { keys: 'Esc', label: 'leave search' },
        ]
      : [
          { keys: '↑↓', label: 'move' },
          { keys: '←→', label: 'switch list' },
          { keys: '↵', label: 'open' },
          { keys: 'Tab', label: 'search' },
          { keys: 'Esc', label: 'close' },
        ],
  );
</script>

{#if uiStore.paletteOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) uiStore.closePalette();
    }}
  >
    <div class="palette" role="dialog" aria-modal="true" aria-label="Go to">
      <NavPanel
        variant="palette"
        onselect={() => uiStore.closePalette()}
        onclose={() => uiStore.closePalette()}
        onsearchfocus={(focused) => (searchFocused = focused)}
      />
      <footer class="legend">
        {#each KEYS as k (k.keys)}
          <span class="legend-item"><kbd>{k.keys}</kbd>{k.label}</span>
        {/each}
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    padding-top: 12vh;
    background: rgba(0, 0, 0, 0.35);
    z-index: 100;
  }

  .palette {
    width: min(560px, 92vw);
    /* Fixed, not fit-content: the whole point is that the box doesn't move
       when the list inside it changes. */
    height: min(520px, 72vh);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
  }

  .legend {
    flex: 0 0 auto;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    padding: 7px 14px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  kbd {
    min-width: 18px;
    padding: 1px 5px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-elevated);
    font-family: var(--font-ui);
    font-size: 10px;
    line-height: 1.5;
    text-align: center;
    color: var(--color-text);
  }

  @media (max-width: 600px) {
    .backdrop {
      padding-top: 6vh;
    }

    .palette {
      height: min(520px, 82vh);
    }
  }
</style>
