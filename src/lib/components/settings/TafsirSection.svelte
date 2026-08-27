<script lang="ts">
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { TafsirView } from '$lib/types/database';
  import SettingRow from './SettingRow.svelte';
  import Segmented from './Segmented.svelte';
  import Toggle from './Toggle.svelte';
  import EditionPicker from '$lib/components/tafsir/EditionPicker.svelte';

  const VIEWS: { value: TafsirView; label: string }[] = [
    { value: 'popover', label: 'Card' },
    { value: 'panel', label: 'Side panel' },
  ];
</script>

<SettingRow
  label="Edition"
  description="Which commentary the reader opens. Changing it here is the same choice the panel's own picker makes."
>
  {#snippet control()}
    {#if tafsirStore.editions.length}
      <!-- The picker sits in the right-hand control column, hard against the
           pane's padding, so its menu has to grow leftwards. -->
      <EditionPicker
        editions={tafsirStore.editions}
        selectedId={tafsirStore.active?.id}
        align="end"
        onselect={(id) => tafsirStore.setEdition(id)}
      />
    {:else}
      <span class="empty">None installed</span>
    {/if}
  {/snippet}
</SettingRow>

<SettingRow
  label="Show commentary in"
  description="A card anchored to the verse, or a panel down the side that follows where you are reading."
  stacked
>
  {#snippet control()}
    <Segmented
      label="Show commentary in"
      options={VIEWS}
      value={tafsirStore.view}
      onchange={(view) => tafsirStore.setView(view)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Tafsir mode"
  description="While it is on, clicking a verse opens its commentary. The toolbar button and the t key toggle the same thing."
>
  {#snippet control()}
    <Toggle
      label="Tafsir mode"
      checked={tafsirStore.clickOpens}
      onchange={(on) => tafsirStore.setClickOpens(on)}
    />
  {/snippet}
</SettingRow>

<p class="pointer">
  Downloading and removing editions lives in
  <button type="button" onclick={() => uiStore.openSettings('editions')}>Editions</button>.
</p>

<style>
  .empty {
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .pointer {
    margin: 16px 0 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }

  .pointer button {
    padding: 0;
    border: none;
    background: none;
    color: var(--color-accent);
    font: inherit;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
</style>
