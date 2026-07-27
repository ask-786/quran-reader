<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap, SvelteSet } from 'svelte/reactivity';
  import { loadPages } from '$lib/api/page-cache';
  import type { Ayah, MushafPage } from '$lib/types/database';
  import {
    ensurePageFont,
    familyForPage,
    loadBasmalaFont,
    trimPageFonts,
  } from '$lib/utils/mushaf-fonts';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
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

  // Al-Fatihah and the opening of Al-Baqarah are set apart in the printed
  // Madani Mushaf: 8 lines instead of 15, each one centred inside the page's
  // decorative frame rather than justified edge-to-edge. Stretching those
  // short lines across the full column (what every other page wants) is what
  // made them read as ordinary body text. The layout data carries no flag for
  // this, so the two pages are named here — the same two the QPC layout marks
  // centred, and it is a property of the printed Mushaf, not of our data.
  const CENTERED_PAGES = new Set([1, 2]);

  // How far either side of the visible pages to render glyphs and load fonts,
  // and how far out to let a page stay rendered before it's dropped again.
  // The gap between the two is hysteresis: without it, a page sitting exactly
  // on the boundary would load and unload on every few pixels of scroll.
  // KEEP_RADIUS also has to outrun auto-scroll, which can cross a page in a
  // couple of seconds — the render radius is what it actually reads from.
  const RENDER_RADIUS = 3;
  const KEEP_RADIUS = 8;

  let pages = $state<MushafPage[]>([]);
  let basmalaFontFamily = $state<string | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let container = $state<HTMLDivElement>();
  let content = $state<HTMLDivElement>();

  /**
   * Pages whose word glyphs are currently in the DOM. Everything outside this
   * set still renders its line boxes — see the `.line` markup below — so the
   * document height, and therefore every scroll position, is identical
   * whether or not a page's glyphs are loaded.
   */
  const renderedPages = new SvelteSet<number>();
  /** The page range `renderedPages` is scoped to. */
  let loadedRangeKey: string | null = null;

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

  /**
   * Ayah id -> the id of the line element its first word sits on. Scroll
   * targets have to resolve through this rather than through a word span:
   * word spans only exist for rendered pages, but every line box always
   * exists, so this lookup works for any Ayah in the range at any time.
   */
  const lineForAyah = $derived.by(() => {
    const map = new SvelteMap<number, string>();
    for (const p of pages) {
      for (const line of p.lines) {
        if (line.line_type !== 'text') continue;
        for (const w of line.words) {
          if (w.ayah_id === null || map.has(w.ayah_id)) continue;
          map.set(w.ayah_id, `line-${p.page}-${line.line_number}`);
        }
      }
    }
    return map;
  });

  function lineAyahId(words: { ayah_id: number | null }[]) {
    return words.find((w) => w.ayah_id !== null)?.ayah_id;
  }

  function updateProgress() {
    if (!container || ayahs.length === 0) return;
    const max = container.scrollHeight - container.clientHeight;
    const fraction = max > 0 ? container.scrollTop / max : 0;
    const idx = Math.min(ayahs.length - 1, Math.max(0, Math.round(fraction * (ayahs.length - 1))));
    const a = ayahs[idx];
    progressStore.update(fraction, a?.juz ?? null, a?.hizb ?? null);
  }

  /**
   * Bring the render window in line with what's on screen: load fonts for
   * pages that have come within RENDER_RADIUS, and drop pages that have
   * fallen outside KEEP_RADIUS. Called from the visibility observer rather
   * than an $effect, so it can write `renderedPages` without depending on it.
   */
  function syncWindow(visible: Set<number>) {
    if (visible.size === 0) return;
    const lo = Math.min(...visible);
    const hi = Math.max(...visible);

    for (
      let p = Math.max(firstPage, lo - RENDER_RADIUS);
      p <= Math.min(lastPage, hi + RENDER_RADIUS);
      p++
    ) {
      if (renderedPages.has(p)) continue;
      const target = p;
      void ensurePageFont(target).then(() => {
        // The window may have moved on, or the view been torn down, while the
        // font was in flight — only commit if the page is still wanted.
        if (target >= lo - KEEP_RADIUS && target <= hi + KEEP_RADIUS) renderedPages.add(target);
      });
    }

    for (const p of renderedPages) {
      if (p < lo - KEEP_RADIUS || p > hi + KEEP_RADIUS) renderedPages.delete(p);
    }

    trimPageFonts(renderedPages);
  }

  /** Which pages the target Ayah needs on screen before it can be scrolled to. */
  function pageOfAyah(ayahId: number | undefined) {
    if (ayahId === undefined) return firstPage;
    return ayahs.find((a) => a.id === ayahId)?.page ?? firstPage;
  }

  $effect(() => {
    const start = firstPage;
    const end = lastPage;
    const targetId = scrollToAyahId;
    loading = true;
    error = null;

    let cancelled = false;
    let observer: IntersectionObserver | undefined;
    let visibility: IntersectionObserver | undefined;

    (async () => {
      try {
        // Layout data for the whole range in one call — it's small (the whole
        // of Al-Baqara is ~6k word rows) and cached across view toggles, so
        // this is what makes the line boxes, and every scroll offset derived
        // from them, correct before any glyph has been fetched.
        const [loaded, basmalaFamily] = await Promise.all([
          loadPages(start, end),
          loadBasmalaFont(),
        ]);
        if (cancelled) return;
        basmalaFontFamily = basmalaFamily;
        pages = loaded;

        // Only a genuinely different page range resets the render window. This
        // effect also re-runs when just the scroll target changes — a Go To
        // inside the open Surah — and clearing there would blank every page on
        // screen and reload the fonts it already had.
        const rangeKey = `${start}-${end}`;
        if (rangeKey !== loadedRangeKey) {
          loadedRangeKey = rangeKey;
          renderedPages.clear();
        }

        // Only the pages around the landing spot are fetched before the first
        // paint. Loading the whole range's fonts up front was the reader's
        // startup stall: 48 files and 5.9MB for Al-Baqara, versus 3 files and
        // ~350KB here. The rest stream in from syncWindow as they're scrolled
        // towards.
        const target = pageOfAyah(targetId);
        const seedLo = Math.max(start, target - 1);
        const seedHi = Math.min(end, target + 1);
        const seeds: number[] = [];
        for (let p = seedLo; p <= seedHi; p++) seeds.push(p);
        await Promise.all(seeds.map((p) => ensurePageFont(p)));
        if (cancelled) return;
        for (const p of seeds) renderedPages.add(p);

        await tick();
        if (cancelled || !container) return;

        if (targetId !== undefined && lineForAyah.has(targetId)) {
          document.getElementById(lineForAyah.get(targetId)!)?.scrollIntoView({ block: 'center' });
        } else {
          container.scrollTo({ top: 0 });
        }

        observer = observeCenteredAyah(
          container,
          container.querySelectorAll('[data-line-ayah-id]'),
          'data-line-ayah-id',
          (id) => (readerPosition.ayahId = id),
        );

        // Drives the render window. The generous rootMargin means a page
        // counts as visible a full viewport before it actually is, giving its
        // font time to arrive before any glyph is needed. IntersectionObserver
        // only reports pages whose state *changed*, so the running set is kept
        // here rather than recomputed from the DOM on every callback.
        // eslint-disable-next-line svelte/prefer-svelte-reactivity -- observer bookkeeping, never rendered
        const visiblePages = new Set<number>();
        visibility = new IntersectionObserver(
          (entries) => {
            for (const entry of entries) {
              const page = Number((entry.target as HTMLElement).dataset.page);
              if (entry.isIntersecting) visiblePages.add(page);
              else visiblePages.delete(page);
            }
            syncWindow(visiblePages);
          },
          { root: container, rootMargin: '100% 0px', threshold: 0 },
        );
        for (const el of container.querySelectorAll('.mushaf-page')) visibility.observe(el);

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
      visibility?.disconnect();
      container?.removeEventListener('scroll', updateProgress);
    };
  });

  /**
   * Reader zoom scales every line box, so the scroll offset that was showing
   * one line points at a different one the moment it changes. Nothing here
   * remounts on a focus toggle — the two modes just carry separate zoom levels
   * — so the view would silently keep an offset that no longer means anything.
   * Re-centring the Ayah the centre tracker last reported is the same
   * record-then-restore round trip a Mushaf/list toggle makes.
   *
   * Only on an actual change: with both modes at the same zoom the current
   * offset is still exact, and re-centring would round it to the nearest line
   * for nothing. `lastZoom` starts unset so the first run (mount) is skipped
   * too — the load effect above does its own landing.
   */
  let lastZoom: number | null = null;

  $effect(() => {
    const zoom = uiStore.readerZoom;
    const changed = lastZoom !== null && zoom !== lastZoom;
    lastZoom = zoom;
    if (!changed) return;

    void (async () => {
      // Focus mode also adds or removes the toolbar and sidebar in this same
      // update; waiting for it keeps the measurement below off a half-applied
      // layout. Nothing read after this point is tracked by the effect.
      await tick();
      if (!container) return;
      const id = readerPosition.ayahId ?? scrollToAyahId;
      const lineId = id == null ? undefined : lineForAyah.get(id);
      if (lineId) document.getElementById(lineId)?.scrollIntoView({ block: 'center' });
      updateProgress();
    })();
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
          <div class="mushaf-page" data-page={p.page} dir="rtl">
            {#each p.lines as line, li (line.line_number)}
              {#if line.line_type === 'surah_header'}
                {@const headerSurah =
                  line.surah_id !== null ? surahsStore.get(line.surah_id) : undefined}
                {@const nextLine = p.lines[li + 1]}
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
                     with the Ayah the line opens on.

                     The line box is always rendered, even for pages outside
                     the glyph window: its height comes from `min-height` and
                     `line-height`, never from its contents, so an empty line
                     and a full one are exactly the same height. That is what
                     lets pages load and unload as you scroll without ever
                     moving the content around them. -->
                <div
                  id="line-{p.page}-{line.line_number}"
                  class="line text-line"
                  class:centered={CENTERED_PAGES.has(p.page)}
                  data-line-ayah-id={lineAyahId(line.words)}
                  style:font-family={familyForPage(p.page)}
                >
                  {#if renderedPages.has(p.page)}
                    {#each line.words as w (w.position)}
                      <span
                        class="word"
                        data-ayah-id={w.ayah_id}
                        aria-label={w.uthmani_text}
                        title={w.uthmani_text}>{w.glyph_v2}</span
                      >
                    {/each}
                  {/if}
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
    /* Query container for .text-line's fit cap below — the lines have to be
       sized against this column, not the viewport. */
    container-type: inline-size;
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
    /* Load-bearing for windowed rendering, not just spacing: this is what
       fixes a line's height independently of whether its glyphs are in the
       DOM. The flex children are single-line spans whose height is 2.5em
       (line-height) — under this floor — so the box measures 2.6em whether
       it holds a full line of Quran or nothing at all. */
    min-height: 2.6em;
  }

  .text-line {
    justify-content: space-between;
    flex-wrap: nowrap;
    /* Same formula as the Ayah list (see ReaderView), so the two views render
       the Quran text at the same size for the same settings — this used to be
       a viewport clamp that ignored --font-size-quran and topped out at 27px,
       so the same Ayah changed size when you toggled views.

       Lines are laid out edge-to-edge like a real Mushaf page and can't
       reflow, so the size is capped at what the column can actually hold.
       The widest line in the Mushaf (page 123, line 8) measures 22.82em of
       glyph advances, so a line fits while font-size <= 100cqi/22.82 =
       4.38cqi; 4.3cqi keeps a margin. At the default 720px column that cap is
       ~31px — above the 28px default, so it only bites on narrow columns,
       where the alternative is horizontal overflow.

       The preceding declaration is the pre-container-query fallback: engines
       without cqi support drop the min() line as invalid and keep it. */
    font-size: calc(clamp(15px, 4.6vw, 27px) * var(--reader-zoom));
    font-size: min(calc(var(--font-size-quran) * var(--reader-zoom)), 4.3cqi);
    line-height: 2.5;
    color: var(--color-text);
  }

  /* Al-Fatihah / the opening of Al-Baqarah (see CENTERED_PAGES): the lines
     keep their printed break points but sit centred, giving the page the
     tapered, rounded block of the printed Mushaf instead of a justified
     rectangle. */
  .text-line.centered {
    justify-content: center;
  }

  /* Justified lines get their word spacing from `space-between`; centred ones
     have none to distribute, and the whitespace between the glyph spans is
     dropped by flex layout, so the gap has to be explicit. Matches the Ayah
     list's 0.35em (see AyahRow) — the QCF fonts' own space is far narrower
     than a live-shaped Arabic font would give. */
  .centered .word {
    margin-inline-end: 0.35em;
  }

  /* Otherwise the trailing gap counts as content and the line centres off by
     half of it. */
  .centered .word:last-child {
    margin-inline-end: 0;
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
