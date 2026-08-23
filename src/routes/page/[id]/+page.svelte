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

<!-- No scroll target: a Mushaf page is a single screen, so opening it already
     puts the reader everywhere there is to be within it. -->
<ReaderPage ayahs={data.ayahs} scope="page" scopeId={data.pageNumber} />
