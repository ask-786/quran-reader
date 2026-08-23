<script lang="ts">
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let { data }: { data: PageData } = $props();

  $effect(() => {
    void data.surah.id;
    autoScrollStore.stop();
    progressStore.reset();
  });
</script>

<svelte:head>
  <title>{data.surah.transliteration} — Quran Reader</title>
</svelte:head>

<ReaderPage
  ayahs={data.ayahs}
  scope="surah"
  scopeId={data.surah.id}
  scrollTarget={data.resumeAyahId}
/>
