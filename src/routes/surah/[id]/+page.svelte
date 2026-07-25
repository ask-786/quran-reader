<script lang="ts">
  import { untrack } from 'svelte';
  import type { PageData } from './$types';
  import ReaderView from '$lib/components/reader/ReaderView.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let { data }: { data: PageData } = $props();

  // Resume from the last-read Ayah only on the initial open of this Surah —
  // must not react to later updates as the reader scrolls (see ReaderView).
  let scrollTarget = $state<number | undefined>();
  $effect(() => {
    const id = data.surah.id;
    scrollTarget = untrack(() =>
      settingsStore.current.last_read_surah_id === id
        ? settingsStore.current.last_read_ayah_id
        : undefined,
    );
  });
</script>

<svelte:head>
  <title>{data.surah.transliteration} — Quran Reader</title>
</svelte:head>

<div class="surah-page">
  <ReaderView
    surah={data.surah}
    ayahs={data.ayahs}
    showAyahNumbers={settingsStore.current.show_ayah_numbers}
    scrollToAyahId={scrollTarget}
  />
</div>

<style>
  .surah-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
</style>
