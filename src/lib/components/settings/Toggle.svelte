<!--
  An on/off switch. A real `<button aria-pressed>` rather than a styled
  checkbox: it is a control that acts immediately, not a field that will be
  submitted, and aria-pressed is what says so.
-->
<script lang="ts">
  let {
    label,
    checked,
    onchange,
  }: {
    label: string;
    checked: boolean;
    onchange: (checked: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  class="toggle"
  class:on={checked}
  aria-pressed={checked}
  aria-label={label}
  onclick={() => onchange(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 38px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: 11px;
    background: var(--color-bg);
    cursor: pointer;
    transition:
      background var(--transition),
      border-color var(--transition);
  }

  .toggle.on {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }

  .toggle:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .knob {
    position: absolute;
    top: 2px;
    /* Driven from the left edge in both states rather than by swapping
       left/right, so the transition has something continuous to animate. */
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-text-muted);
    transition:
      transform var(--transition),
      background var(--transition);
  }

  .toggle.on .knob {
    transform: translateX(16px);
    background: var(--color-accent-contrast);
  }
</style>
