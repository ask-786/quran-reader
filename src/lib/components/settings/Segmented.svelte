<!--
  A row of mutually exclusive choices, all visible at once.

  A radio group rather than a `<select>` or a cycling button: these are two- to
  four-way choices whose options are the whole point (three themes, three column
  widths), and a control that hides two of three options behind a click is how
  the toolbar's blind theme cycle got here.
-->
<script lang="ts" generics="T extends string">
  let {
    label,
    options,
    value,
    onchange,
  }: {
    /** Names the group for assistive tech; the visible label lives in the row. */
    label: string;
    options: { value: T; label: string }[];
    value: T;
    onchange: (value: T) => void;
  } = $props();

  /**
   * Roving arrow keys, the radio-group pattern. Selection follows the cursor
   * here, which is what a radio group is meant to do and is harmless because
   * every option is applied instantly and reversibly.
   */
  function onKeydown(e: KeyboardEvent) {
    const forward = e.key === 'ArrowRight' || e.key === 'ArrowDown';
    const back = e.key === 'ArrowLeft' || e.key === 'ArrowUp';
    if (!forward && !back) return;
    e.preventDefault();
    // Stopped as well as consumed: the reader's global shortcuts sit on
    // `window` and read bare arrows as page turns.
    e.stopPropagation();
    const i = options.findIndex((o) => o.value === value);
    const next = (i + (forward ? 1 : -1) + options.length) % options.length;
    onchange(options[next].value);
    const group = e.currentTarget as HTMLElement;
    group.querySelectorAll('button')[next]?.focus();
  }
</script>

<!-- tabindex on the group itself is never reached — the radios inside it are
     what take focus — but the role is an interactive one and the a11y rule
     asks for it regardless. -->
<div class="segmented" role="radiogroup" tabindex="-1" aria-label={label} onkeydown={onKeydown}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      class="segment"
      class:on={option.value === value}
      aria-checked={option.value === value}
      tabindex={option.value === value ? 0 : -1}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: flex;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
  }

  .segment {
    flex: 1;
    padding: 5px 12px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background var(--transition),
      color var(--transition);
  }

  .segment:hover:not(.on) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .segment.on {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .segment:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }
</style>
