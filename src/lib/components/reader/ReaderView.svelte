<script lang="ts">
  import { tick } from 'svelte';
  import type { Ayah } from '$lib/types/database';
  import AyahRow from './AyahRow.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let {
    surahId,
    ayahs,
    translations = {},
    showTranslation = true,
    showAyahNumbers = true,
    scrollToAyahId,
  }: {
    surahId: number;
    ayahs: Ayah[];
    translations?: Record<number, string>;
    showTranslation?: boolean;
    showAyahNumbers?: boolean;
    scrollToAyahId?: number;
  } = $props();

  let container = $state<HTMLDivElement>();
  let lastReadTimer: ReturnType<typeof setTimeout>;

  function pageChanged(i: number) {
    return i > 0 && ayahs[i].page !== ayahs[i - 1].page;
  }
  function juzChanged(i: number) {
    return i > 0 && ayahs[i].juz !== ayahs[i - 1].juz;
  }

  function onIntersect(entries: IntersectionObserverEntry[]) {
    const visible = entries
      .filter((e) => e.isIntersecting)
      .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
    if (!visible) return;
    const id = Number(visible.target.id.replace('ayah-', ''));
    if (!id) return;
    clearTimeout(lastReadTimer);
    lastReadTimer = setTimeout(() => settingsStore.setLastRead(surahId, id), 400);
  }

  // Re-run scroll positioning and intersection tracking whenever the ayah list changes (surah navigation).
  $effect(() => {
    const current = ayahs;
    const targetId = scrollToAyahId;
    let observer: IntersectionObserver | undefined;

    (async () => {
      await tick();
      if (!container) return;

      if (targetId) {
        document.getElementById(`ayah-${targetId}`)?.scrollIntoView({ block: 'center' });
      } else {
        container.scrollTo({ top: 0 });
      }

      observer = new IntersectionObserver(onIntersect, { root: container, threshold: [0.6] });
      container.querySelectorAll('[id^="ayah-"]').forEach((el) => observer!.observe(el));
    })();

    void current;
    return () => observer?.disconnect();
  });
</script>

<div bind:this={container} class="reader-scroll scrollbar-thin">
  {#each ayahs as ayah, i (ayah.id)}
    {#if pageChanged(i)}
      <div class="boundary-divider">
        <span
          >Page {ayah.page}{#if juzChanged(i)}
            · Juz {ayah.juz}{/if}</span
        >
      </div>
    {/if}
    <AyahRow
      {ayah}
      translation={showTranslation ? (translations[ayah.id] ?? null) : null}
      {showAyahNumbers}
    />
  {/each}
</div>

<style>
  .reader-scroll {
    flex: 1;
    overflow-y: auto;
    scroll-behavior: smooth;
    padding: 0 max(24px, calc((100% - var(--reader-max-width)) / 2)) 80px;
  }

  .boundary-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin: 20px 0;
    color: var(--color-text-faint);
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .boundary-divider::before,
  .boundary-divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }
</style>
