<script lang="ts">
  import { untrack } from 'svelte';
  import { page } from '$app/stores';
  import type { PageData } from './$types';
  import ReaderView from '$lib/components/reader/ReaderView.svelte';
  import PageView from '$lib/components/reader/PageView.svelte';
  import AutoScrollHandle from '$lib/components/reader/AutoScrollHandle.svelte';
  import ProgressIndicator from '$lib/components/reader/ProgressIndicator.svelte';
  import ReaderZoomControl from '$lib/components/reader/ReaderZoomControl.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let { data }: { data: PageData } = $props();

  // Resume from the last-read Ayah, or jump to a `?ayah=` deep link, only on
  // the initial open of this Surah — must not react to later updates as the
  // reader scrolls (see ReaderView).
  let scrollTarget = $state<number | undefined>();
  $effect(() => {
    const id = data.surah.id;
    const ayahParam = $page.url.searchParams.get('ayah');
    const linked = ayahParam
      ? data.ayahs.find((a) => a.ayah_number === Number(ayahParam))
      : undefined;
    scrollTarget =
      linked?.id ??
      untrack(() =>
        settingsStore.current.last_read_surah_id === id
          ? settingsStore.current.last_read_ayah_id
          : undefined,
      );
    autoScrollStore.stop();
    progressStore.reset();
  });
</script>

<svelte:head>
  <title>{data.surah.transliteration} — Quran Reader</title>
</svelte:head>

<div class="surah-page">
  {#if uiStore.readingMode === 'mushaf'}
    <PageView ayahs={data.ayahs} scrollToAyahId={scrollTarget} />
  {:else}
    <ReaderView
      surah={data.surah}
      ayahs={data.ayahs}
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
  .surah-page {
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
