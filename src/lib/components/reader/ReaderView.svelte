<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import type { Ayah, AyahGlyphWord, GlyphSpan, Surah } from '$lib/types/database';
  import { getPage } from '$lib/api/db';
  import { loadPageFonts, loadBasmalaFont } from '$lib/utils/mushaf-fonts';
  import AyahRow from './AyahRow.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { observeCenteredAyah } from '$lib/utils/centered-ayah';

  let {
    ayahs,
    translations = {},
    showTranslation = true,
    scrollToAyahId,
  }: {
    ayahs: Ayah[];
    translations?: Record<number, string>;
    showTranslation?: boolean;
    scrollToAyahId?: number;
  } = $props();

  // Ayah lists can span multiple Surahs (Juz/Hizb browsing), so a header is
  // rendered per contiguous run of one Surah's Ayahs, keyed by its start index.
  // A Juz/Hizb/Page range can also open partway into a Surah (Juz 2 begins at
  // Al-Baqara 142), and a banner + Bismillah there would announce an opening
  // that isn't one — so a run only gets a header if it starts at Ayah 1.
  const segments = $derived.by(() => {
    const map = new SvelteMap<number, { surah?: Surah }>();
    let segStart = 0;
    for (let i = 1; i <= ayahs.length; i++) {
      if (i === ayahs.length || ayahs[i].surah_id !== ayahs[segStart].surah_id) {
        if (ayahs[segStart].ayah_number === 1) {
          map.set(segStart, { surah: surahsStore.get(ayahs[segStart].surah_id) });
        }
        segStart = i;
      }
    }
    return map;
  });

  let container = $state<HTMLDivElement>();
  let content = $state<HTMLDivElement>();
  let lastReadTimer: ReturnType<typeof setTimeout>;

  // Page-glyph words per Ayah, fetched from the same page data PageView uses,
  // so list-mode text matches Mushaf/page-mode rendering quality instead of
  // live-shaping ayah.uthmani_text through the buggy system font stack.
  const ayahWords = new SvelteMap<number, AyahGlyphWord[]>();
  // Basmala glyph words per Surah id, from the same fetched pages — a Surah's
  // basmala line always immediately follows its surah_header line on the
  // page that opens it, so it's within the same page range as its Ayahs.
  const basmalaWords = new SvelteMap<number, GlyphSpan[]>();
  let basmalaFontFamily = $state<string | null>(null);
  // Rows render empty until the glyph words below have been fetched, so they
  // collapse to a fraction of their real height. Scrolling to an Ayah before
  // then measures those stub rows and lands nowhere near it — positioning waits
  // on this flag.
  let wordsReady = $state(false);

  const firstPage = $derived(ayahs[0]?.page ?? 1);
  const lastPage = $derived(ayahs[ayahs.length - 1]?.page ?? firstPage);

  $effect(() => {
    const start = firstPage;
    const end = lastPage;
    let cancelled = false;
    wordsReady = false;

    (async () => {
      const pageNumbers = Array.from({ length: end - start + 1 }, (_, i) => start + i);
      const [pageData, fontFamilies, basmalaFamily] = await Promise.all([
        Promise.all(pageNumbers.map((p) => getPage(p))),
        loadPageFonts(pageNumbers),
        loadBasmalaFont(),
      ]);
      if (cancelled) return;
      basmalaFontFamily = basmalaFamily;

      ayahWords.clear();
      basmalaWords.clear();
      let pendingHeaderSurahId: number | null = null;
      for (const data of pageData) {
        const fontFamily = fontFamilies.get(data.page) ?? null;
        for (const line of data.lines) {
          if (line.line_type === 'surah_header') {
            pendingHeaderSurahId = line.surah_id;
            continue;
          }
          if (line.line_type === 'basmala') {
            if (pendingHeaderSurahId !== null) basmalaWords.set(pendingHeaderSurahId, line.words);
            pendingHeaderSurahId = null;
            continue;
          }
          if (line.line_type !== 'text') continue;
          for (const w of line.words) {
            if (w.ayah_id === null) continue;
            const word: AyahGlyphWord = {
              uthmani_text: w.uthmani_text,
              glyph_v2: w.glyph_v2,
              fontFamily,
            };
            const list = ayahWords.get(w.ayah_id);
            if (list) list.push(word);
            else ayahWords.set(w.ayah_id, [word]);
          }
        }
      }
      wordsReady = true;
    })();

    return () => {
      cancelled = true;
    };
  });

  function pageChanged(i: number) {
    return i > 0 && ayahs[i].page !== ayahs[i - 1].page;
  }
  function juzChanged(i: number) {
    return i > 0 && ayahs[i].juz !== ayahs[i - 1].juz;
  }

  function onCenteredAyah(id: number) {
    // Published immediately so a Mushaf/list toggle can pick it up, while
    // last-read stays debounced — that one hits the settings store on disk.
    readerPosition.ayahId = id;
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

  // Whether this instance has already placed itself once. The first placement
  // is a mount — resuming a Surah, or the Mushaf/list toggle rebuilding this
  // view — and jumping there instantly is right; only later target changes
  // (Go To within the open Surah) read as navigation worth easing.
  let hasPositioned = false;

  // Re-run scroll positioning and centre tracking whenever the ayah list changes (surah navigation).
  $effect(() => {
    const current = ayahs;
    const targetId = scrollToAyahId;
    if (!wordsReady) return;
    let observer: IntersectionObserver | undefined;

    (async () => {
      await tick();
      if (!container) return;

      if (targetId) {
        document
          .getElementById(`ayah-${targetId}`)
          ?.scrollIntoView({ block: 'center', behavior: hasPositioned ? 'smooth' : 'auto' });
      } else {
        container.scrollTo({ top: 0 });
      }
      hasPositioned = true;

      observer = observeCenteredAyah(
        container,
        container.querySelectorAll('.ayah-row'),
        'data-ayah-id',
        onCenteredAyah,
      );

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
          <SurahHeader
            surah={seg.surah}
            basmalaWords={basmalaWords.get(seg.surah.id) ?? []}
            {basmalaFontFamily}
          />
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
        words={ayahWords.get(ayah.id) ?? []}
        translation={showTranslation ? (translations[ayah.id] ?? null) : null}
      />
    {/each}
  </div>
</div>

<style>
  .reader-scroll {
    flex: 1;
    overflow-y: auto;
    /* Not scroll-behavior: smooth — that CSS-wide smoothing fights the
       auto-scroll loop below, which drives its own per-frame scrollTop
       increments and needs those applied instantly (see PageView, which
       has never had this rule and auto-scrolls smoothly as a result).
       The one spot that wants an eased jump (scrollIntoView on ayah
       navigation) requests { behavior: 'smooth' } itself instead. */
    /* The reading column grows with reader zoom (capped at 100% so it can
       never force horizontal overflow) instead of staying pinned to the
       narrow/normal/wide preset width, so the wide idle margins shrink as
       the text gets bigger rather than staying empty. */
    padding: 0
      max(
        var(--reader-side-padding),
        calc((100% - min(100%, var(--reader-max-width) * var(--reader-zoom))) / 2)
      )
      80px;
  }

  /* Reader-only zoom: scales just the Quran text within this view, layered
     on top of the global font-size setting. Applied directly to font-size
     (not by redeclaring --font-size-quran in terms of itself, which is a
     cyclic custom-property reference and gets silently dropped as invalid). */
  .reader-scroll :global(.quran-text) {
    font-size: calc(var(--font-size-quran) * var(--reader-zoom));
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
