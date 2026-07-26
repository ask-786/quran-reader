<script lang="ts">
  import { tick } from 'svelte';
  import type { Ayah, Surah } from '$lib/types/database';
  import AyahRow from './AyahRow.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let {
    surah,
    ayahs,
    translations = {},
    showTranslation = true,
    showAyahNumbers = true,
    scrollToAyahId,
  }: {
    surah: Surah;
    ayahs: Ayah[];
    translations?: Record<number, string>;
    showTranslation?: boolean;
    showAyahNumbers?: boolean;
    scrollToAyahId?: number;
  } = $props();

  const rukuCount = $derived(ayahs.length ? ayahs[ayahs.length - 1].ruku - ayahs[0].ruku + 1 : 0);

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
    lastReadTimer = setTimeout(() => settingsStore.setLastRead(surah.id, id), 400);
  }

  function updateProgress() {
    if (!container || ayahs.length === 0) return;
    const max = container.scrollHeight - container.clientHeight;
    const fraction = max > 0 ? container.scrollTop / max : 0;
    const idx = Math.min(ayahs.length - 1, Math.max(0, Math.round(fraction * (ayahs.length - 1))));
    const a = ayahs[idx];
    progressStore.update(fraction, a?.juz ?? null, a?.hizb ?? null);
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

      container.addEventListener('scroll', updateProgress, { passive: true });
      updateProgress();
    })();

    void current;
    return () => {
      observer?.disconnect();
      container?.removeEventListener('scroll', updateProgress);
    };
  });

  $effect(() => {
    if (!autoScrollStore.active || !container) return;
    let raf: number;
    let last = performance.now();

    function step(now: number) {
      const dt = (now - last) / 1000;
      last = now;
      if (container) {
        const dy = autoScrollStore.tick(dt);
        if (dy !== 0) container.scrollTop += dy;
        if (container.scrollTop + container.clientHeight >= container.scrollHeight - 1) {
          autoScrollStore.stop();
          return;
        }
      }
      raf = requestAnimationFrame(step);
    }
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div bind:this={container} class="reader-scroll scrollbar-thin">
  <SurahHeader {surah} {rukuCount} />
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
    /* Reader-only zoom: scales just the Quran text within this view, layered
       on top of the global font-size setting. Scoped by re-declaring the
       same custom property so every .quran-text descendant picks it up. */
    --font-size-quran: calc(var(--font-size-quran) * var(--reader-zoom));
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
