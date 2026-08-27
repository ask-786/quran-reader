<script lang="ts">
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Theme } from '$lib/types/database';
  import SettingRow from './SettingRow.svelte';
  import Segmented from './Segmented.svelte';

  const THEMES: { value: Theme; label: string }[] = [
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' },
    { value: 'sepia', label: 'Sepia' },
  ];

  const ZOOM_STEPS = [0.7, 0.8, 0.9, 1, 1.1, 1.25, 1.5];
  const appZoom = $derived(settingsStore.current.app_zoom);
</script>

<SettingRow label="Theme" description="Applies to the whole app, not only the reader.">
  {#snippet control()}
    <Segmented
      label="Theme"
      options={THEMES}
      value={settingsStore.current.theme}
      onchange={(theme) => settingsStore.setTheme(theme)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="App zoom"
  description="Scales every part of the interface — chrome, lists and text alike. Ctrl + and Ctrl − do the same thing from anywhere."
  stacked
>
  {#snippet control()}
    <div class="zoom">
      <!-- Fixed stops rather than a slider: the webview renders at whatever
           factor it is given, and the round ones are the ones whose text lands
           on whole pixels. -->
      {#each ZOOM_STEPS as stepValue (stepValue)}
        <button
          type="button"
          class="stop"
          class:on={Math.abs(appZoom - stepValue) < 0.001}
          onclick={() => settingsStore.setAppZoom(stepValue)}
        >
          {Math.round(stepValue * 100)}%
        </button>
      {/each}
    </div>
  {/snippet}
</SettingRow>

<style>
  .zoom {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .stop {
    padding: 5px 10px;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    background: var(--color-bg);
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition:
      background var(--transition),
      color var(--transition);
  }

  .stop:hover:not(.on) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .stop.on {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-accent-contrast);
  }
</style>
