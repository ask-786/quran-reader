<script lang="ts">
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let { data }: { data: PageData } = $props();

  $effect(() => {
    void data.pageNumber;
    uiStore.readingMode = 'mushaf';
    autoScrollStore.stop();
    progressStore.reset(`Page ${data.pageNumber}`);
  });
</script>

<svelte:head>
  <title>Page {data.pageNumber} — Quran Reader</title>
</svelte:head>

<ReaderPage ayahs={data.ayahs} />
