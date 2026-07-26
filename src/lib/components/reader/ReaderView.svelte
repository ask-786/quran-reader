<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import type { Ayah, Surah } from '$lib/types/database';
  import AyahRow from './AyahRow.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';

  let {
    ayahs,
    translations = {},
    showTranslation = true,
    showAyahNumbers = true,
    scrollToAyahId,
  }: {
    ayahs: Ayah[];
    translations?: Record<number, string>;
    showTranslation?: boolean;
    showAyahNumbers?: boolean;
    scrollToAyahId?: number;
  } = $props();

  // Ayah lists can span multiple Surahs (Juz/Hizb browsing), so a header is
  // rendered per contiguous run of one Surah's Ayahs, keyed by its start index.
  const segments = $derived.by(() => {
    const map = new SvelteMap<number, { surah?: Surah; rukuCount: number }>();
    let segStart = 0;
    for (let i = 1; i <= ayahs.length; i++) {
      if (i === ayahs.length || ayahs[i].surah_id !== ayahs[segStart].surah_id) {
        const first = ayahs[segStart];
        const last = ayahs[i - 1];
        map.set(segStart, {
          surah: surahsStore.get(first.surah_id),
          rukuCount: last.ruku - first.ruku + 1,
        });
        segStart = i;
      }
    }
    return map;
  });

  let container = $state<HTMLDivElement>();
  let content = $state<HTMLDivElement>();
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
    const ayah = ayahs.find((a) => a.id === id);
    if (!ayah) return;
    clearTimeout(lastReadTimer);
    lastReadTimer = setTimeout(() => settingsStore.setLastRead(ayah.surah_id, id), 400);
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
        const { whole, fraction } = autoScrollStore.tick(dt);
        if (whole !== 0) container.scrollTop += whole;
        if (content) content.style.transform = fraction > 0 ? `translateY(${-fraction}px)` : '';
        if (container.scrollTop + container.clientHeight >= container.scrollHeight - 1) {
          autoScrollStore.stop();
          return;
        }
      }
      raf = requestAnimationFrame(step);
    }
    raf = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(raf);
      if (content) content.style.transform = '';
    };
  });
</script>

<div bind:this={container} class="reader-scroll scrollbar-none">
  <div bind:this={content} class="reader-content">
    {#each ayahs as ayah, i (ayah.id)}
      {#if segments.has(i)}
        {@const seg = segments.get(i)}
        {#if seg?.surah}
          <SurahHeader surah={seg.surah} rukuCount={seg.rukuCount} />
        {/if}
      {/if}
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
</div>

<style>
  .reader-scroll {
    flex: 1;
    overflow-y: auto;
    scroll-behavior: smooth;
    padding: 0 max(var(--reader-side-padding), calc((100% - var(--reader-max-width)) / 2)) 80px;
    /* Reader-only zoom: scales just the Quran text within this view, layered
       on top of the global font-size setting. Scoped by re-declaring the
       same custom property so every .quran-text descendant picks it up. */
    --font-size-quran: calc(var(--font-size-quran) * var(--reader-zoom));
  }

  .reader-content {
    /* Auto-scroll drives a sub-pixel translateY on this wrapper each
       frame to smooth out low-speed scrolling (see auto-scroll.svelte.ts);
       it stays a no-op transform otherwise. */
    will-change: transform;
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
