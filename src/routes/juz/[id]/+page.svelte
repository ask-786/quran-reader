<script lang="ts">
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let { data }: { data: PageData } = $props();

  $effect(() => {
    void data.juz;
    autoScrollStore.stop();
    progressStore.reset(`Juz ${data.juz}`);
  });
</script>

<svelte:head>
  <title>Juz {data.juz} — Quran Reader</title>
</svelte:head>

<ReaderPage ayahs={data.ayahs} />
