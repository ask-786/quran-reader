<!--
  A continuous value with its number shown beside it.

  A native range input, not a pair of step buttons like the zoom controls: font
  size spans 27 steps and leading 15, and stepping to the far end of either
  through a button is not a thing anyone will do. The native control also
  arrives with its own keyboard handling, which is the part hand-rolled sliders
  usually lose.
-->
<script lang="ts">
  let {
    label,
    value,
    min,
    max,
    step,
    format,
    oninput,
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    /** How the number reads — "28 px", "2.2×". The raw value rarely says
     *  enough on its own. */
    format: (value: number) => string;
    oninput: (value: number) => void;
  } = $props();
</script>

<div class="slider">
  <input
    type="range"
    aria-label={label}
    {min}
    {max}
    {step}
    {value}
    oninput={(e) => oninput(Number(e.currentTarget.value))}
  />
  <output class="readout">{format(value)}</output>
</div>

<style>
  .slider {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  input[type='range'] {
    flex: 1;
    min-width: 0;
    /* Colours the native track and thumb from the theme in one declaration.
       Engines without it fall back to the platform slider, which is usable —
       just not themed. */
    accent-color: var(--color-accent);
  }

  input[type='range']:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 4px;
    border-radius: 2px;
  }

  .readout {
    flex-shrink: 0;
    min-width: 52px;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    text-align: end;
    color: var(--color-text-muted);
  }
</style>
