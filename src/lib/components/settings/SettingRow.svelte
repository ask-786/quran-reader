<!--
  One preference: what it is, what it does, and the control that changes it.

  The description is not decoration. Several of these settings interact with
  something the reader can see elsewhere — reader zoom multiplies the font
  size, verse markers survive in the Mushaf view — and a label alone cannot say
  so, which is how a setting ends up looking broken.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    description,
    /** Renders the control beneath the label instead of beside it. For controls
     *  too wide to sit in a right-hand column — a segmented control of three,
     *  a slider — where squeezing them would cost more than the row height. */
    stacked = false,
    control,
  }: {
    label: string;
    description?: string;
    stacked?: boolean;
    control: Snippet;
  } = $props();
</script>

<div class="row" class:stacked>
  <div class="text">
    <span class="label">{label}</span>
    {#if description}<span class="description">{description}</span>{/if}
  </div>
  <div class="control">{@render control()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 0;
  }

  .row + :global(.row) {
    border-top: 1px solid var(--color-border);
  }

  .row.stacked {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
    color: var(--color-text);
  }

  .description {
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .control {
    flex-shrink: 0;
  }

  .row.stacked .control {
    width: 100%;
  }
</style>
