<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap, SvelteSet } from 'svelte/reactivity';
  import type { Ayah, AyahGlyphWord, GlyphSpan, Surah } from '$lib/types/database';
  import { loadPages } from '$lib/api/page-cache';
  import {
    ensurePageFont,
    familyForPage,
    loadBasmalaFont,
    loadFontMap,
    trimPageFonts,
  } from '$lib/utils/mushaf-fonts';
  import AyahRow from './AyahRow.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';
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

  // See PageView for the reasoning behind the two radii.
  const RENDER_RADIUS = 3;
  const KEEP_RADIUS = 8;

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
  // on this flag. It also holds back SurahHeader's live-shaped Bismillah
  // fallback, which would otherwise paint and then swap to the QCF glyphs.
  let wordsReady = $state(false);

  /** Pages whose Ayah rows currently have their glyph spans in the DOM. */
  const renderedPages = new SvelteSet<number>();

  /**
   * Measured pixel height of each rendered row, so a row that leaves the
   * window can hold exactly the space it occupied. Unlike the Mushaf view —
   * where a line box is a fixed 2.6em whatever it contains — list rows wrap,
   * so their height genuinely depends on their content and has to be observed
   * rather than derived.
   */
  // Deliberately a plain Map, not a SvelteMap: it's written on every row
  // measurement, and making those writes reactive would re-render the whole
  // list each time. Reserved heights are read during a render that some other
  // reactive change (renderedPages) has already triggered, so the latest
  // values are always picked up without this needing to signal anything.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const rowHeights = new Map<number, number>();
  /**
   * Mean height per word across measured rows, used to size rows that have
   * never been rendered. Reactive, so the very first measurements replace the
   * initial guess for every row still waiting — but only republished when it
   * moves materially, since each publish re-renders the whole list.
   */
  let avgHeightPerWord = $state(0);
  let runningAvg = 0;
  let measuredRows = 0;
  /** The page range the glyph maps below were built for. */
  let loadedRangeKey: string | null = null;

  const firstPage = $derived(ayahs[0]?.page ?? 1);
  const lastPage = $derived(ayahs[ayahs.length - 1]?.page ?? firstPage);

  function wordCount(ayahId: number) {
    return ayahWords.get(ayahId)?.length ?? 0;
  }

  /** Reserved height for a row whose glyphs aren't rendered. */
  function reservedHeight(ayahId: number): number {
    const measured = rowHeights.get(ayahId);
    if (measured !== undefined) return measured;
    if (avgHeightPerWord > 0) return Math.max(48, wordCount(ayahId) * avgHeightPerWord);
    return 96;
  }

  /** Fold one row's real height into the table and the running mean. */
  function measureRow(ayahId: number, height: number) {
    if (height <= 0) return;
    rowHeights.set(ayahId, height);
    const words = wordCount(ayahId);
    if (words === 0) return;
    measuredRows++;
    runningAvg += (height / words - runningAvg) / measuredRows;
  }

  /**
   * Republish the mean if it has drifted materially. This resizes every
   * unmeasured row at once — including the ones above the viewport — so it has
   * to be bracketed by withStableScroll. Left uncompensated it walks the view
   * off whatever it was looking at, which is precisely what a Mushaf/list
   * toggle used to do the instant the first measurements arrived.
   */
  function publishAvg() {
    if (measuredRows === 0) return;
    const drift =
      avgHeightPerWord === 0
        ? Infinity
        : Math.abs(runningAvg - avgHeightPerWord) / avgHeightPerWord;
    if (drift > 0.05) void withStableScroll(() => (avgHeightPerWord = runningAvg));
  }

  /**
   * Measure the rows already on screen and publish the mean *before* the view
   * positions itself. Without this, first paint sizes every unrendered row
   * with the fallback guess — a flat 96px for Ayahs that run from one word to
   * 128 — and the measurements landing a moment later reflow the entire list
   * under a view that has already scrolled to its target. Applied straight
   * rather than through withStableScroll: there is no scroll position worth
   * preserving yet, and deferring it would let the landing happen first.
   */
  function primeHeights() {
    if (!container) return;
    for (const el of container.querySelectorAll<HTMLElement>('.ayah-row')) {
      const id = Number(el.dataset.ayahId);
      const page = Number(el.dataset.page);
      if (Number.isFinite(id) && renderedPages.has(page)) measureRow(id, el.offsetHeight);
    }
    if (measuredRows > 0) avgHeightPerWord = runningAvg;
  }

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

  /**
   * The row nearest the top of the viewport, used to hold the view still
   * across a window change. Reserved heights are estimates until a row has
   * been rendered once, so filling in or dropping rows *above* the viewport
   * shifts everything below by the estimation error. Re-pinning this row
   * afterwards absorbs that shift, which is what keeps scrolling smooth
   * instead of visibly lurching as the window moves.
   */
  function anchorRow(): { el: Element; top: number } | null {
    if (!container) return null;
    const rootTop = container.getBoundingClientRect().top;
    let best: Element | null = null;
    let bestTop = 0;
    let bestD = Infinity;
    for (const el of container.querySelectorAll('.ayah-row')) {
      const top = el.getBoundingClientRect().top;
      const d = Math.abs(top - rootTop);
      if (d < bestD) {
        bestD = d;
        best = el;
        bestTop = top;
      }
    }
    return best ? { el: best, top: bestTop } : null;
  }

  /**
   * Run a change that alters reserved row heights while holding the view
   * still: capture the anchor, apply, let the DOM settle, then put the anchor
   * back where it was. Every such change goes through here — pages entering
   * and leaving the window, and the mean-height republish alike.
   *
   * Rounds are chained rather than allowed to overlap. Two in flight would
   * each capture an anchor and each correct scrollTop for the same shift,
   * applying it twice.
   */
  let anchorChain: Promise<void> = Promise.resolve();

  function withStableScroll(apply: () => void): Promise<void> {
    const round = anchorChain.then(async () => {
      if (!container) {
        apply();
        return;
      }
      const anchor = anchorRow();
      apply();
      await tick();
      if (anchor && container?.contains(anchor.el)) {
        container.scrollTop += anchor.el.getBoundingClientRect().top - anchor.top;
      }
    });
    // A failed round must not poison every round after it.
    anchorChain = round.catch(() => {});
    return round;
  }

  /**
   * The Ayah this view was told to land on, held until the first window sync
   * has settled. Landing happens against estimated heights for everything
   * outside the seeded pages; re-asserting once those have been filled in and
   * measured makes the final position exact rather than merely close.
   */
  let unsettledTargetId: number | null = null;

  function reassertTarget() {
    if (unsettledTargetId === null || !container) return;
    const el = document.getElementById(`ayah-${unsettledTargetId}`);
    unsettledTargetId = null;
    el?.scrollIntoView({ block: 'center', behavior: 'auto' });
  }

  // syncWindow awaits font loads, so scrolling can fire another round before
  // the last one has committed. Two overlapping runs would each capture an
  // anchor and each correct scrollTop, double-counting the same shift — so
  // rounds are serialised, with only the newest pending request kept.
  let syncing = false;
  let pendingVisible: Set<number> | null = null;

  async function syncWindow(visible: Set<number>) {
    if (syncing) {
      pendingVisible = new Set(visible);
      return;
    }
    syncing = true;
    try {
      await syncWindowOnce(visible);
      while (pendingVisible) {
        const next = pendingVisible;
        pendingVisible = null;
        await syncWindowOnce(next);
      }
      // The window is now filled in around the landing spot, so this is the
      // first moment the target's position is backed by real heights.
      await anchorChain;
      reassertTarget();
    } finally {
      syncing = false;
    }
  }

  async function syncWindowOnce(visible: Set<number>) {
    if (visible.size === 0 || !container) return;
    const lo = Math.min(...visible);
    const hi = Math.max(...visible);

    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- local scratch set, never rendered
    const wanted = new Set<number>();
    for (
      let p = Math.max(firstPage, lo - RENDER_RADIUS);
      p <= Math.min(lastPage, hi + RENDER_RADIUS);
      p++
    ) {
      wanted.add(p);
    }

    const toAdd = [...wanted].filter((p) => !renderedPages.has(p));
    const toDrop = [...renderedPages].filter((p) => p < lo - KEEP_RADIUS || p > hi + KEEP_RADIUS);
    if (toAdd.length === 0 && toDrop.length === 0) return;

    await Promise.all(toAdd.map((p) => ensurePageFont(p)));
    if (!container) return;

    await withStableScroll(() => {
      for (const p of toAdd) renderedPages.add(p);
      for (const p of toDrop) renderedPages.delete(p);
    });
    trimPageFonts(renderedPages);
  }

  $effect(() => {
    const start = firstPage;
    const end = lastPage;
    const targetId = scrollToAyahId;
    let cancelled = false;
    wordsReady = false;

    (async () => {
      // One call for the whole range, served from the process-wide cache on a
      // view toggle or a revisit — this is what makes flipping between the
      // Mushaf and the list instant instead of refetching every page.
      const [pageData, basmalaFamily] = await Promise.all([
        loadPages(start, end),
        loadBasmalaFont(),
        loadFontMap(),
      ]);
      if (cancelled) return;
      basmalaFontFamily = basmalaFamily;

      // Only a genuinely different page range invalidates the glyph maps and
      // the measured heights. This effect also re-runs when just the scroll
      // target changes — a Go To inside the open Surah — and resetting there
      // would drop every rendered page and blank the view before rebuilding
      // exactly what it already had.
      const rangeKey = `${start}-${end}`;
      if (rangeKey !== loadedRangeKey) {
        loadedRangeKey = rangeKey;
        ayahWords.clear();
        basmalaWords.clear();
        renderedPages.clear();
        rowHeights.clear();
        avgHeightPerWord = 0;
        runningAvg = 0;
        measuredRows = 0;
      } else if (ayahWords.size > 0) {
        // Same range, maps already built — just re-seed around the new target.
        const targetPage = targetId ? (ayahs.find((a) => a.id === targetId)?.page ?? start) : start;
        for (let p = Math.max(start, targetPage - 1); p <= Math.min(end, targetPage + 1); p++) {
          await ensurePageFont(p);
          if (cancelled) return;
          renderedPages.add(p);
        }
        wordsReady = true;
        return;
      }

      let pendingHeaderSurahId: number | null = null;
      for (const data of pageData) {
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
              glyph_v4: w.glyph_v4,
              // The family name comes from font-map.json (loaded above) and
              // the font itself loads lazily, so this can be filled in
              // before the file has arrived.
              fontFamily: familyForPage(data.page) ?? null,
            };
            const list = ayahWords.get(w.ayah_id);
            if (list) list.push(word);
            else ayahWords.set(w.ayah_id, [word]);
          }
        }
      }

      // Seed only the pages around the landing spot; the rest stream in as
      // they're scrolled towards. See PageView for the cost this avoids.
      const targetPage = targetId ? (ayahs.find((a) => a.id === targetId)?.page ?? start) : start;
      const seeds: number[] = [];
      for (let p = Math.max(start, targetPage - 1); p <= Math.min(end, targetPage + 1); p++) {
        seeds.push(p);
      }
      await Promise.all(seeds.map((p) => ensurePageFont(p)));
      if (cancelled) return;
      for (const p of seeds) renderedPages.add(p);

      wordsReady = true;
    })();

    return () => {
      cancelled = true;
    };
  });

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
    let visibility: IntersectionObserver | undefined;
    let sizes: ResizeObserver | undefined;

    (async () => {
      await tick();
      if (!container) return;

      // Size the unrendered rows from real measurements before landing, so the
      // scroll below is computed against heights that won't shift underneath it.
      primeHeights();
      await tick();
      if (!container) return;

      if (targetId) {
        document
          .getElementById(`ayah-${targetId}`)
          ?.scrollIntoView({ block: 'center', behavior: hasPositioned ? 'smooth' : 'auto' });
        unsettledTargetId = targetId;
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

      // Records the real height of every row while it's rendered, so the same
      // row can reserve exactly that space once it leaves the window.
      sizes = new ResizeObserver((entries) => {
        for (const entry of entries) {
          const el = entry.target as HTMLElement;
          const id = Number(el.dataset.ayahId);
          const page = Number(el.dataset.page);
          // offsetHeight, not entry.contentRect: `box-sizing: border-box` is
          // global (app.css), so the height we later set back on the row is a
          // border-box height and has to be measured as one — contentRect
          // excludes the row's padding and would reserve too little.
          if (Number.isFinite(id) && renderedPages.has(page)) measureRow(id, el.offsetHeight);
        }
        publishAvg();
      });
      for (const el of container.querySelectorAll('.ayah-row')) sizes.observe(el);

      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- observer bookkeeping, never rendered
      const visiblePages = new Set<number>();
      visibility = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            const page = Number((entry.target as HTMLElement).dataset.page);
            if (entry.isIntersecting) visiblePages.add(page);
            else visiblePages.delete(page);
          }
          void syncWindow(visiblePages);
        },
        { root: container, rootMargin: '100% 0px', threshold: 0 },
      );
      for (const el of container.querySelectorAll('.ayah-row')) visibility.observe(el);

      container.addEventListener('scroll', updateProgress, { passive: true });
      updateProgress();
    })();

    void current;
    return () => {
      observer?.disconnect();
      visibility?.disconnect();
      sizes?.disconnect();
      container?.removeEventListener('scroll', updateProgress);
    };
  });

  /**
   * Reader zoom scales the Quran text in every row, so the scroll offset that
   * was showing one Ayah points at a different one the moment it changes. This
   * view isn't remounted on a focus toggle — the two modes just carry separate
   * zoom levels — so it would silently keep an offset that no longer means
   * anything. Re-centring the Ayah the centre tracker last reported is the
   * same record-then-restore round trip a Mushaf/list toggle makes.
   *
   * Only on an actual change: with both modes at the same zoom the current
   * offset is still exact, and re-centring would round it to the nearest row
   * for nothing. `lastZoom` starts unset so the first run (mount) is skipped
   * too — the positioning effect above does its own landing.
   */
  let lastZoom: number | null = null;

  $effect(() => {
    const zoom = uiStore.readerZoom;
    const changed = lastZoom !== null && zoom !== lastZoom;
    lastZoom = zoom;
    if (!changed) return;

    void (async () => {
      // Focus mode also adds or removes the toolbar and sidebar in this same
      // update; waiting for it keeps the measurements below off a half-applied
      // layout. Nothing read after this point is tracked by the effect.
      await tick();
      // A window round already in flight would otherwise restore its own
      // anchor on top of the landing below.
      await anchorChain;
      if (!container) return;

      // Every reserved height in the table was measured at the old zoom, and
      // the rows holding them are the ones off screen — leaving them would put
      // the target's neighbours at the wrong distance and drift the view as
      // they scroll back in. Re-measure what's on screen at the new zoom and
      // republish the mean before landing, exactly as first paint does; the
      // rows still rendered keep their natural height throughout, so nothing
      // falls back to the crude 96px guess in between.
      rowHeights.clear();
      runningAvg = 0;
      measuredRows = 0;
      primeHeights();
      await tick();
      if (!container) return;

      const id = readerPosition.ayahId ?? scrollToAyahId;
      if (id != null) {
        document.getElementById(`ayah-${id}`)?.scrollIntoView({ block: 'center' });
      }
      updateProgress();
    })();
  });

  // Page jumps and top/bottom shortcuts drive this view's scroller from the
  // layout's key handler and the jump buttons; rows carry the page they belong
  // to, so a page boundary resolves to the first row tagged with it.
  $effect(() => {
    if (!container) return;
    return readerScroll.register(container, '.ayah-row');
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
            glyphsPending={!wordsReady}
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
        words={renderedPages.has(ayah.page) ? (ayahWords.get(ayah.id) ?? []) : []}
        reservedHeight={renderedPages.has(ayah.page) ? null : reservedHeight(ayah.id)}
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
