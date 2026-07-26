<script lang="ts">
  import type { Ayah } from '$lib/types/database';
  import ReaderView from './ReaderView.svelte';
  import PageView from './PageView.svelte';
  import AutoScrollHandle from './AutoScrollHandle.svelte';
  import ProgressIndicator from './ProgressIndicator.svelte';
  import ReaderZoomControl from './ReaderZoomControl.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let {
    ayahs,
    scrollTarget,
  }: {
    ayahs: Ayah[];
    scrollTarget?: number;
  } = $props();
</script>

<div class="reader-page">
  {#if uiStore.readingMode === 'mushaf'}
    <PageView {ayahs} scrollToAyahId={scrollTarget} />
  {:else}
    <ReaderView
      {ayahs}
      showAyahNumbers={settingsStore.current.show_ayah_numbers}
      scrollToAyahId={scrollTarget}
    />
  {/if}
  <ProgressIndicator />
  <AutoScrollHandle />
  <div class="zoom-control-slot">
    <ReaderZoomControl />
  </div>
</div>

<style>
  .reader-page {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .zoom-control-slot {
    position: absolute;
    top: 16px;
    right: 30px;
    z-index: 6;
  }
</style>
