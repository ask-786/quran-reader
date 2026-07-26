<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { getPage } from '$lib/api/db';
  import type { Ayah, MushafPage } from '$lib/types/database';
  import { loadPageFonts, loadBasmalaFont } from '$lib/utils/mushaf-fonts';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { observeCenteredAyah } from '$lib/utils/centered-ayah';
  import SurahHeader from './SurahHeader.svelte';

  let {
    ayahs,
    scrollToAyahId,
  }: {
    ayahs: Ayah[];
    scrollToAyahId?: number;
  } = $props();

  type LoadedPage = { page: number; data: MushafPage; fontFamily: string | null };

  let pages = $state<LoadedPage[]>([]);
  let basmalaFontFamily = $state<string | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let container = $state<HTMLDivElement>();
  let content = $state<HTMLDivElement>();

  const firstPage = $derived(ayahs[0]?.page ?? 1);
  const lastPage = $derived(ayahs[ayahs.length - 1]?.page ?? firstPage);

  // Page -> Juz, derived from this Surah's own Ayahs. Pages that also carry a
  // neighbouring Surah's content are still keyed off what we know, matching
  // how ReaderView derives its boundary dividers.
  const juzByPage = $derived.by(() => {
    const map = new SvelteMap<number, number>();
    for (const a of ayahs) if (!map.has(a.page)) map.set(a.page, a.juz);
    return map;
  });

  function updateProgress() {
    if (!container || ayahs.length === 0) return;
    const max = container.scrollHeight - container.clientHeight;
    const fraction = max > 0 ? container.scrollTop / max : 0;
    const idx = Math.min(ayahs.length - 1, Math.max(0, Math.round(fraction * (ayahs.length - 1))));
    const a = ayahs[idx];
    progressStore.update(fraction, a?.juz ?? null, a?.hizb ?? null);
  }

  $effect(() => {
    const start = firstPage;
    const end = lastPage;
    const targetId = scrollToAyahId;
    loading = true;
    error = null;
    pages = [];

    let cancelled = false;
    let observer: IntersectionObserver | undefined;

    (async () => {
      try {
        const basmalaFamily = await loadBasmalaFont();
        if (cancelled) return;
        basmalaFontFamily = basmalaFamily;

        const pageNumbers = Array.from({ length: end - start + 1 }, (_, i) => start + i);
        const [pageData, fontFamilies] = await Promise.all([
          Promise.all(pageNumbers.map((p) => getPage(p))),
          loadPageFonts(pageNumbers),
        ]);
        if (cancelled) return;
        pages = pageNumbers.map((p, i) => ({
          page: p,
          data: pageData[i],
          fontFamily: fontFamilies.get(p) ?? null,
        }));

        await tick();
        if (cancelled || !container) return;
        if (targetId) {
          container
            .querySelector(`[data-ayah-id="${targetId}"]`)
            ?.scrollIntoView({ block: 'center' });
        } else {
          container.scrollTo({ top: 0 });
        }

        observer = observeCenteredAyah(
          container,
          container.querySelectorAll('[data-line-ayah-id]'),
          'data-line-ayah-id',
          (id) => (readerPosition.ayahId = id),
        );

        container.addEventListener('scroll', updateProgress, { passive: true });
        updateProgress();
      } catch (err) {
        if (!cancelled) error = err instanceof Error ? err.message : String(err);
      } finally {
        if (!cancelled) loading = false;
      }
    })();

    return () => {
      cancelled = true;
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

<div class="page-view">
  <div bind:this={container} class="page-surface scrollbar-none">
    {#if error}
      <p class="state-message">Couldn't load pages: {error}</p>
    {:else if loading && pages.length === 0}
      <p class="state-message">Loading…</p>
    {:else}
      <div bind:this={content} class="page-content">
        {#each pages as p, i (p.page)}
          {#if i > 0}
            <div class="boundary-divider">
              <span
                >Page {p.page}{#if juzByPage.get(p.page) !== juzByPage.get(pages[i - 1].page)}
                  · Juz {juzByPage.get(p.page)}{/if}</span
              >
            </div>
          {/if}
          <div class="mushaf-page" dir="rtl">
            {#each p.data.lines as line, li (line.line_number)}
              {#if line.line_type === 'surah_header'}
                {@const headerSurah =
                  line.surah_id !== null ? surahsStore.get(line.surah_id) : undefined}
                {@const nextLine = p.data.lines[li + 1]}
                {#if headerSurah}
                  <SurahHeader
                    surah={headerSurah}
                    basmalaWords={nextLine?.line_type === 'basmala' ? nextLine.words : []}
                    {basmalaFontFamily}
                  />
                {/if}
              {:else if line.line_type === 'basmala'}
                <!-- rendered as part of the preceding surah_header, above -->
              {:else}
                <!-- Individual words are too many to observe (thousands per
                     Surah), so centre tracking runs a line at a time, tagged
                     with the Ayah the line opens on. -->
                {@const lineAyahId = line.words.find((w) => w.ayah_id !== null)?.ayah_id}
                <div
                  class="line text-line"
                  data-line-ayah-id={lineAyahId}
                  style:font-family={p.fontFamily}
                >
                  {#each line.words as w (w.position)}
                    <span
                      class="word"
                      data-ayah-id={w.ayah_id}
                      aria-label={w.uthmani_text}
                      title={w.uthmani_text}>{w.glyph_v2}</span
                    >
                  {/each}
                </div>
              {/if}
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .page-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .page-surface {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px var(--reader-side-padding) 80px;
  }

  .page-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    /* Auto-scroll drives a sub-pixel translateY on this wrapper each
       frame to smooth out low-speed scrolling (see auto-scroll.svelte.ts);
       it stays a no-op transform otherwise. */
    will-change: transform;
  }

  /* Reader-only zoom: scale the page width and glyph font-sizes together
     (via calc, not the CSS `zoom` property) so justified lines stay
     proportional. Plain calc()/font-size keeps normal scroll-height
     calculation intact — `zoom` broke overflow-y:auto in WebKitGTK.
     Width is driven by the same --reader-max-width as the aya list view
     (not a separate hardcoded value), so both views land on the same
     column width at any given zoom level. */
  .mushaf-page {
    width: calc(min(100%, var(--reader-max-width) * var(--reader-zoom)));
  }

  .boundary-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: calc(min(100%, var(--reader-max-width) * var(--reader-zoom)));
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

  .line {
    display: flex;
    align-items: baseline;
    min-height: 2.6em;
  }

  .text-line {
    justify-content: space-between;
    flex-wrap: nowrap;
    /* Lines are laid out edge-to-edge like a real Mushaf page and can't
       reflow, so on narrow screens the glyphs shrink with the viewport
       instead of overflowing into horizontal scroll. */
    font-size: calc(clamp(15px, 4.6vw, 27px) * var(--reader-zoom));
    line-height: 2.5;
    color: var(--color-text);
  }

  .word {
    cursor: default;
  }

  .word:hover {
    color: var(--color-accent);
  }

  .state-message {
    color: var(--color-text-muted);
    font-size: 14px;
    text-align: center;
    margin-top: 40px;
  }
</style>
