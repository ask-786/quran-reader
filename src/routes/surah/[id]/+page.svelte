<script lang="ts">
  import { untrack } from 'svelte';
  import { page } from '$app/stores';
  import type { PageData } from './$types';
  import ReaderPage from '$lib/components/reader/ReaderPage.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
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

<ReaderPage ayahs={data.ayahs} {scrollTarget} />
