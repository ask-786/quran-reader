<script lang="ts">
  import { ChevronsUp, ChevronsDown } from 'lucide-svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';
</script>

<div class="jump-group" title="Jump to start or end">
  <button
    class="icon-btn"
    onclick={() => readerScroll.scrollToTop()}
    aria-label="Jump to the start"
  >
    <ChevronsUp size={14} />
  </button>
  <button
    class="icon-btn"
    onclick={() => readerScroll.scrollToBottom()}
    aria-label="Jump to the end"
  >
    <ChevronsDown size={14} />
  </button>
</div>

<style>
  /* Deliberately the same shell as the zoom control it sits under, so the two
     read as one stack of reader controls rather than two floating widgets. */
  .jump-group {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    box-sizing: border-box;
    width: 100%;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 22px;
    padding: 0;
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
</style>
