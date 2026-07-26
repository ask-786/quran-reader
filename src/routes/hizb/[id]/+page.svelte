<script lang="ts">
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let { data }: { data: PageData } = $props();

  $effect(() => {
    void data.hizb;
    autoScrollStore.stop();
    progressStore.reset(`Hizb ${data.hizb}`);
  });
</script>

<svelte:head>
  <title>Hizb {data.hizb} — Quran Reader</title>
</svelte:head>

<ReaderPage ayahs={data.ayahs} />
