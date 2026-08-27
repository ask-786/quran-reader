<!--
  The typography of the Quran text itself. Every setting here was already in
  the database and already wired to a CSS custom property both reading views
  consume — this is the control that was missing.
-->
<script lang="ts">
  import { RotateCcw } from 'lucide-svelte';
  import {
    settingsStore,
    FONT_SIZE_MIN,
    FONT_SIZE_MAX,
    FONT_SIZE_STEP,
    LINE_HEIGHT_MIN,
    LINE_HEIGHT_MAX,
    LINE_HEIGHT_STEP,
  } from '$lib/stores/settings.svelte';
  import type { ReaderWidth } from '$lib/types/database';
  import SettingRow from './SettingRow.svelte';
  import Segmented from './Segmented.svelte';
  import Slider from './Slider.svelte';
  import Toggle from './Toggle.svelte';
  import TypographySpecimen from './TypographySpecimen.svelte';

  const WIDTHS: { value: ReaderWidth; label: string }[] = [
    { value: 'narrow', label: 'Narrow' },
    { value: 'normal', label: 'Normal' },
    { value: 'wide', label: 'Wide' },
  ];

  const current = $derived(settingsStore.current);

  const atDefaults = $derived(
    current.font_size === 28 && current.line_height === 2.2 && current.reader_width === 'normal',
  );
</script>

<TypographySpecimen />

<SettingRow
  label="Text size"
  description="The size the Quran is set at, in both views. The reader's own zoom control multiplies this rather than replacing it."
  stacked
>
  {#snippet control()}
    <Slider
      label="Quran text size"
      value={current.font_size}
      min={FONT_SIZE_MIN}
      max={FONT_SIZE_MAX}
      step={FONT_SIZE_STEP}
      format={(v) => `${v} px`}
      oninput={(v) => settingsStore.setFontSize(v)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Line spacing"
  description="Room between lines, as a multiple of the text size. Vocalised Arabic needs more of it than Latin does."
  stacked
>
  {#snippet control()}
    <Slider
      label="Line spacing"
      value={current.line_height}
      min={LINE_HEIGHT_MIN}
      max={LINE_HEIGHT_MAX}
      step={LINE_HEIGHT_STEP}
      format={(v) => `${v.toFixed(1)}×`}
      oninput={(v) => settingsStore.setLineHeight(v)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Column width"
  description="How wide the text column runs before it wraps. Both views use the same measure."
  stacked
>
  {#snippet control()}
    <Segmented
      label="Column width"
      options={WIDTHS}
      value={current.reader_width}
      onchange={(width) => settingsStore.setReaderWidth(width)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Verse numbers"
  description="The end-of-verse marker in the scrolling view. Mushaf page view always keeps it — its lines are justified around the printed page's own glyphs, marker included."
>
  {#snippet control()}
    <Toggle
      label="Show verse numbers"
      checked={current.show_ayah_numbers}
      onchange={(show) => settingsStore.setShowAyahNumbers(show)}
    />
  {/snippet}
</SettingRow>

<div class="reset">
  <button type="button" disabled={atDefaults} onclick={() => settingsStore.resetTypography()}>
    <RotateCcw size={13} />
    Reset typography
  </button>
</div>

<style>
  .reset {
    display: flex;
    justify-content: flex-end;
    padding-top: 14px;
  }

  .reset button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .reset button:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .reset button:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
