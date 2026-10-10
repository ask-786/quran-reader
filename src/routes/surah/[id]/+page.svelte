<script lang="ts">
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';
  import { routinesStore } from '$lib/stores/routines.svelte';
  import { untrack } from 'svelte';

  let { data }: { data: PageData } = $props();

  $effect(() => {
    void data.surah.id;
    autoScrollStore.stop();
    progressStore.reset();
  });

  // Opened as a step of a routine: enter it now that the navigation has really
  // happened. Not in the load function, which also runs to preload a link the
  // pointer is only resting on.
  $effect(() => {
    const routine = data.routine;
    const surahId = data.surah.id;
    if (!routine) return;
    untrack(() => void routinesStore.enter(routine.id, surahId, routine.restart));
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
