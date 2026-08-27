<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteMap, SvelteSet } from 'svelte/reactivity';
  import { loadPages } from '$lib/api/page-cache';
  import type { Ayah, MushafPage, RangeFocus } from '$lib/types/database';
  import {
    ensurePageFont,
    familyForPage,
    loadBasmalaFont,
    loadFontMap,
    trimPageFonts,
  } from '$lib/utils/mushaf-fonts';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { progressStore } from '$lib/stores/progress.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';
  import { observeCenteredAyah } from '$lib/utils/centered-ayah';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import SurahHeader from './SurahHeader.svelte';

  let {
    ayahs,
    scrollToAyahId,
    rangeFocus = 'all',
  }: {
    ayahs: Ayah[];
    scrollToAyahId?: number;
    /**
     * What to do with the parts of a page outside the open range. A printed
     * page is shared — open Al-Mulk and the sheet it starts on carries the last
     * lines of Al-Mulk's predecessor above the banner — so those lines can be
     * kept, dimmed to context, or dropped.
     *
     * Decided by the caller, not read from settings here, because it is only
     * meaningful for a range that *can* be a fraction of a page. See ReaderPage
     * for why the Mushaf page route always asks for `all`.
     */
    rangeFocus?: RangeFocus;
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
  /**
   * Whether the page content is fit to be painted — which means the layout is
   * loaded *and* the glyphs around the landing spot are registered, not just
   * the first of those.
   *
   * Layout arrives well before the ~760KB font group it needs, and a line box
   * takes its height from `min-height`, not its contents, so between the two
   * the range renders at full height with every line empty. On a cold open
   * that empty sheet is on screen long enough to read as a flash of blank page
   * before the text drops in. Holding the whole thing back until the seed
   * glyphs are in trades that for a moment longer on "Loading…" — one state
   * instead of two.
   *
   * Not simply "the load effect is running": that effect also re-runs for a
   * new scroll target inside the range the reader is already looking at (a Go
   * To), and blanking the view to jump within it would be a far worse flash
   * than the one being fixed. Only a genuinely different range clears this.
   */
  let ready = $state(false);
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

  /**
   * Whether an Ayah is one of the ones this route opened. Every range the
   * reader covers is a contiguous run of global Ayah ids, so the two ends of
   * `ayahs` are the whole test.
   *
   * Words with no Ayah of their own — a basmala line's single glyph — count as
   * in-range: they belong to the Surah banner above them, which is judged
   * separately by `croppedSurahHeader`.
   */
  function inRange(ayahId: number | null): boolean {
    if (ayahId === null || ayahs.length === 0) return true;
    return ayahId >= ayahs[0].id && ayahId <= ayahs[ayahs.length - 1].id;
  }

  /** The Surahs that actually *begin* inside the open range. */
  const openingSurahs = $derived.by(() => {
    const ids = new SvelteSet<number>();
    for (const a of ayahs) if (a.ayah_number === 1) ids.add(a.surah_id);
    return ids;
  });

  /**
   * A Surah banner belongs to the range only when its Surah opens inside it.
   * The banner at the foot of the last page announces the *next* Surah, which
   * is exactly what this is for; a banner at the head of the first page
   * announces the one you opened, so it stays.
   *
   * Judged by the Surah's own Ayah 1 rather than by "any Ayah of this Surah is
   * in range", so a Juz that opens partway into a Surah doesn't keep a banner
   * for an opening it doesn't contain.
   */
  function headerOutOfRange(surahId: number | null): boolean {
    if (rangeFocus === 'all') return false;
    return surahId === null || !openingSurahs.has(surahId);
  }

  /**
   * Text lines with nothing of the open range on them. Ids rather than a
   * predicate called per line during render: this runs once per page load
   * instead of once per line per render, and the render loop is the hot path
   * the whole view's scrolling performance sits on.
   *
   * Surah boundaries always fall between lines — a Surah begins on the line
   * after its banner and the next banner ends it — so for a Surah route this
   * is the whole job. A Juz or Hizb boundary can land mid-line, and there the
   * line stays and its individual out-of-range words are marked instead (see
   * `.out-of-range` in the markup below).
   */
  const outOfRangeLines = $derived.by(() => {
    const outside = new SvelteSet<string>();
    if (rangeFocus === 'all' || ayahs.length === 0) return outside;
    for (const p of pages) {
      for (const line of p.lines) {
        if (line.line_type !== 'text') continue;
        if (line.words.some((w) => w.ayah_id !== null && inRange(w.ayah_id))) continue;
        outside.add(`line-${p.page}-${line.line_number}`);
      }
    }
    return outside;
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

  /**
   * Page -> the Juz that finished on it, read by that page's own footer.
   * Labelled by the Juz it closes rather than the one it opens: a page number
   * reports the page you just finished, so a Juz sharing that rule has to
   * report the Juz you just finished too — one row pointing two directions at
   * once ("21 · Juz 2", meaning page 21 ended and Juz 2 begins) is the kind of
   * thing a reader has to stop and decode.
   *
   * Twenty-six of the thirty end exactly at a page boundary. The other four end
   * partway down a page (Juz 3 after line 1 of page 62, Juz 6 after line 11 of
   * 121, Juz 10 after line 12 of 201, Juz 25 after line 8 of 502), and those
   * roll *forward* onto the foot of the page they ended on rather than getting
   * a rule of their own: a rule cutting between two lines breaks up a page that
   * should read as one printed sheet, and the ۞ ornament at the head of the
   * following line already marks the exact spot, as it does in print.
   *
   * Forward is the only direction that stays true. By the foot of page 62 Juz 3
   * really is finished, so the row is late but never wrong; hanging it on page
   * 61's footer instead would announce a Juz as complete while a line of it was
   * still to come.
   *
   * The loaded range is a contiguous run of Ayahs, so a Juz change from one
   * Ayah to the next is a true boundary and no bookkeeping beyond the previous
   * value is needed. It also means a range opening mid-Juz never claims a
   * boundary it didn't cross.
   */
  const juzCompletedOnPage = $derived.by(() => {
    const map = new SvelteMap<number, number>();

    const pageOfLine = new SvelteMap<string, number>();
    // A page's first *text* line, so a Juz opening after a Surah header and its
    // Basmala still counts as opening the page rather than interrupting it.
    const opensPage = new SvelteSet<string>();
    for (const p of pages) {
      let seenText = false;
      for (const l of p.lines) {
        if (l.line_type !== 'text') continue;
        const id = `line-${p.page}-${l.line_number}`;
        pageOfLine.set(id, p.page);
        if (!seenText) {
          opensPage.add(id);
          seenText = true;
        }
      }
    }

    let prev = ayahs[0]?.juz;
    for (const a of ayahs) {
      if (a.juz === prev) continue;
      const completed = prev;
      prev = a.juz;
      const line = lineForAyah.get(a.id);
      if (completed === undefined || line === undefined) continue;
      const page = pageOfLine.get(line);
      if (page === undefined) continue;
      // A boundary that opens a page closed the page before it; one that lands
      // mid-page closed nothing yet — that page's own foot is the first honest
      // place to say so.
      map.set(opensPage.has(line) ? page - 1 : page, completed);
    }
    return map;
  });

  /**
   * Tap a word, get its Ayah's commentary — plan item 11.3's word-level
   * trigger.
   *
   * Delegated from the scroll container rather than bound per word: a page can
   * carry hundreds of word spans and Al-Baqara's range runs to thousands, and
   * this is the render path the whole Mushaf view's performance sits on.
   *
   * An imperative listener via an action, not `onclick` on the div, because
   * this element is a scroll surface and not a control — giving it a button
   * role and a key handler to satisfy the a11y lint would be a lie about what
   * it is. The keyboard route to the same thing is `t`, which opens on the
   * Ayah the reader is already on.
   */
  function tafsirOnWordClick(node: HTMLElement) {
    const onClick = (e: MouseEvent) => {
      // A click ending a drag-selection is a copy, not a request.
      if (!window.getSelection()?.isCollapsed) return;
      const target = e.target as HTMLElement | null;
      const word = target?.closest<HTMLElement>('.word');
      // Basmala words carry no Ayah of their own, so the line they sit on is
      // the fallback — it is tagged with the Ayah the line opens on.
      const line = target?.closest<HTMLElement>('[data-line-ayah-id]');
      const raw = word?.dataset.ayahId ?? line?.dataset.lineAyahId;
      const ayahId = Number(raw);
      if (!raw || !Number.isFinite(ayahId)) return;
      tafsirStore.openFromClick(ayahId, word ?? line ?? null);
    };
    node.addEventListener('click', onClick);
    return {
      destroy() {
        node.removeEventListener('click', onClick);
      },
    };
  }

  /**
   * The Ayah a line is tagged with for centre tracking. On a line the range
   * only starts partway through — a Juz boundary, or the continuation of an
   * Ayah that began on the previous page — the opening word belongs to an Ayah
   * outside the range, and tagging the line with that publishes a position
   * ReaderPage can only throw away. The first word actually in range is the
   * honest answer, and the only one the reader can be sent back to.
   */
  function lineAyahId(words: { ayah_id: number | null }[]) {
    const scoped = words.find((w) => w.ayah_id !== null && inRange(w.ayah_id));
    return (scoped ?? words.find((w) => w.ayah_id !== null))?.ayah_id;
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
    error = null;

    // Decided synchronously, before anything is awaited, so the view is only
    // ever blanked for a range it isn't already showing. The window is cleared
    // here for the same reason it is held back — both describe the outgoing
    // range, and neither should survive into the incoming one.
    const rangeKey = `${start}-${end}`;
    if (rangeKey !== loadedRangeKey) {
      loadedRangeKey = rangeKey;
      renderedPages.clear();
      ready = false;
    }

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
          loadFontMap(),
        ]);
        if (cancelled) return;
        basmalaFontFamily = basmalaFamily;
        pages = loaded;

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
        // Same update as the glyphs above, so the page's first paint is a page
        // with text on it. `tick()` then lands the scroll before that paint —
        // it resolves on the DOM update, still inside this task's microtasks.
        ready = true;

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
   * Reader zoom scales every line box, and trimming adds or removes whole
   * ones, so either way the scroll offset that was showing one line points at
   * a different one the moment it changes. Nothing here remounts on a focus
   * toggle or a settings change — the two modes just carry separate zoom
   * levels, and the page-focus control lives in a dialog over the reader — so
   * the view would silently keep an offset that no longer means anything.
   * Re-centring the Ayah the centre tracker last reported is the same
   * record-then-restore round trip a Mushaf/list toggle makes.
   *
   * Only on an actual change: with both modes at the same zoom the current
   * offset is still exact, and re-centring would round it to the nearest line
   * for nothing. Dimming is tracked as "does this mode trim" for the same
   * reason — going between `all` and `dim` recolours the page without moving a
   * single line. Both trackers start unset so the first run (mount) is skipped
   * too — the load effect above does its own landing.
   */
  let lastZoom: number | null = null;
  let lastTrims: boolean | null = null;

  $effect(() => {
    const zoom = uiStore.readerZoom;
    const trims = rangeFocus === 'trim';
    const changed =
      (lastZoom !== null && zoom !== lastZoom) || (lastTrims !== null && trims !== lastTrims);
    lastZoom = zoom;
    lastTrims = trims;
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

  // Page jumps and top/bottom shortcuts drive this view's scroller from the
  // layout's key handler and the jump buttons. Here a page boundary is simply
  // the page box itself.
  $effect(() => {
    if (!container) return;
    return readerScroll.register(container, '.mushaf-page');
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
  <div
    bind:this={container}
    class="page-surface scrollbar-none"
    class:tafsir-clickable={tafsirStore.clicksOpenTafsir}
    use:tafsirOnWordClick
  >
    {#if error}
      <p class="state-message">Couldn't load pages: {error}</p>
    {:else if !ready}
      <p class="state-message">Loading…</p>
    {:else}
      <div bind:this={content} class="page-content" class:dim={rangeFocus === 'dim'}>
        {#each pages as p (p.page)}
          <div class="mushaf-page" data-page={p.page} dir="rtl">
            {#each p.lines as line, li (line.line_number)}
              {#if line.line_type === 'surah_header'}
                {@const headerSurah =
                  line.surah_id !== null ? surahsStore.get(line.surah_id) : undefined}
                {@const nextLine = p.lines[li + 1]}
                {@const bannerOutside = headerOutOfRange(line.surah_id)}
                <!-- The wrapper is what carries the out-of-range mark: the
                     banner is a component, and a class it doesn't know about
                     has to land on something in this file. It adds no box of
                     its own — no padding or border, so the banner's margins
                     collapse through it exactly as before. -->
                {#if headerSurah && !(rangeFocus === 'trim' && bannerOutside)}
                  <div class="banner-slot" class:out-of-range={bannerOutside}>
                    <SurahHeader
                      surah={headerSurah}
                      basmalaWords={nextLine?.line_type === 'basmala' ? nextLine.words : []}
                      {basmalaFontFamily}
                    />
                  </div>
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
                {@const lineId = `line-${p.page}-${line.line_number}`}
                {@const lineOutside = outOfRangeLines.has(lineId)}
                <div
                  id={lineId}
                  class="line text-line"
                  class:centered={CENTERED_PAGES.has(p.page)}
                  class:out-of-range={lineOutside}
                  data-line-ayah-id={lineAyahId(line.words)}
                  style:font-family={familyForPage(p.page)}
                >
                  {#if renderedPages.has(p.page)}
                    {#each line.words as w (w.position)}
                      <!-- `data-word-index` is written but not yet read. Word
                           level actions (root, morphology) are a known future
                           feature and they need this attribute on this exact
                           hot path; adding it now costs one attribute and
                           saves touching the render loop again. Which gesture
                           they get is decided in
                           docs/tafsir-popover-plan.md — not this one. -->
                      <span
                        class="word"
                        class:out-of-range={!lineOutside &&
                          rangeFocus !== 'all' &&
                          !inRange(w.ayah_id)}
                        data-ayah-id={w.ayah_id}
                        data-word-index={w.word_index}
                        aria-label={w.uthmani_text}
                        title={w.uthmani_text}>{w.glyph_v4}</span
                      >
                    {/each}
                  {/if}
                </div>
              {/if}
            {/each}
            <!-- Inside the page box, not between pages: the printed Mushaf
                 puts the folio number at the foot of the page it numbers.
                 Every page gets one, including the first of the range — the
                 old between-pages divider was skipped at i === 0, so opening
                 a Surah mid-Mushaf left its first page unnumbered.

                 When a Juz finished on this page, the row carries that as well
                 rather than letting the Juz draw a marker of its own. Both
                 halves report the same thing: what you have just finished. -->
            <div class="page-footer" class:juz-boundary={juzCompletedOnPage.has(p.page)} dir="ltr">
              <span>{p.page}</span>
              {#if juzCompletedOnPage.has(p.page)}
                <span class="sep">·</span>
                <span class="juz-no">Juz {juzCompletedOnPage.get(p.page)} complete</span>
              {/if}
            </div>
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

  /* Was carried by the old between-pages divider's margin; now that the page
     box owns its own footer, the box owns the separation too. */
  .mushaf-page + .mushaf-page {
    margin-top: 28px;
  }

  .page-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    /* Spaced in the page's own em, so it tracks reader zoom with the text
       above it instead of drifting as the column grows. */
    margin-top: 0.6em;
    color: var(--color-text-faint);
    font-family: var(--font-ui);
    font-size: 11px;
    letter-spacing: 0.06em;
  }

  .page-footer::before,
  .page-footer::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }

  /* Same row, same rule — only the colour changes, so a Juz boundary reads as
     a page boundary that matters rather than as a different kind of thing. */
  .page-footer.juz-boundary::before,
  .page-footer.juz-boundary::after {
    background: color-mix(in srgb, var(--color-accent) 40%, transparent);
  }

  .juz-no {
    color: var(--color-accent);
    text-transform: uppercase;
  }

  .line {
    display: flex;
    align-items: baseline;
    /* Load-bearing for windowed rendering, not just spacing: this is what
       fixes a line's height independently of whether its glyphs are in the
       DOM. The flex children are single-line spans whose height is 2.5em
       (line-height, below) — under this floor — so the box measures 2.6em
       whether it holds a full line of Quran or nothing at all. */
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
       The widest-line cap below (4.3cqi) was measured against QCF v2's glyph
       advances (page 123, line 8: 22.82em) — v4 claims narrower glyphs (its
       own docs cite removed inter-word gaps), so this cap is carried over
       unverified rather than loosened; it can only bind tighter than
       necessary, never overflow. Needs re-measuring against v4 glyphs.

       The preceding declaration is the pre-container-query fallback: engines
       without cqi support drop the min() line as invalid and keep it. */
    font-size: calc(clamp(15px, 4.6vw, 27px) * var(--reader-zoom));
    font-size: min(calc(var(--font-size-quran) * var(--reader-zoom)), 4.3cqi);
    /* Carried unchanged from QCF v2: measuring line pitch off rendered v2 and
       v4 pages put them within 0.6% at the same size, so v4's taller declared
       metrics don't want more leading here. See --line-height-quran in
       app.css. Keep in step with .line's min-height above. */
    line-height: 2.5;
    color: var(--color-text);
  }

  /* The rest of the printed page — see `rangeFocus`. Trim is the default and
     is expressed as the absence of `.dim` on the content root, so a page with
     nothing out of range costs no extra rules either way.

     Whole lines and the Surah banner go out of flow: the point of trimming is
     that the Surah you opened starts at the top of its page, which a run of
     reserved-but-empty lines above the banner would not give you. The line box
     stays in the DOM so the centre tracker and the visibility observer keep the
     element references they were set up with — a display:none box simply never
     intersects. */
  .line.out-of-range,
  .banner-slot.out-of-range {
    display: none;
  }

  /* The partial-line case, which only a Juz or Hizb boundary produces (Surahs
     always break between lines). Hidden in place rather than removed: a
     justified line gets its word spacing from `space-between`, so dropping a
     word would re-space the whole line and stop it being the printed line. */
  .word.out-of-range {
    visibility: hidden;
  }

  /* Dim: the same three marks, kept on the page as context. Opacity rather
     than a faint colour so one number covers the Quran glyphs, the banner's
     accent rules and its subtitle alike, and reads the same in every theme.
     `pointer-events` is what stops a click on faint text opening commentary
     for a verse the reader did not open — the delegated handler finds no
     `.word` and no tagged line, so it does nothing. */
  .page-content.dim .line.out-of-range,
  .page-content.dim .banner-slot.out-of-range {
    display: flex;
    opacity: 0.26;
    pointer-events: none;
  }

  .page-content.dim .banner-slot.out-of-range {
    display: block;
  }

  .page-content.dim .word.out-of-range {
    visibility: visible;
    opacity: 0.26;
    pointer-events: none;
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

  /* Mirrors .ayah-text.clickable in AyahRow: the cursor is the whole of what
     tells you tafsir mode is on. */
  .tafsir-clickable .word {
    cursor: pointer;
  }

  .word {
    cursor: default;
    /* QCF v4's Private Use Area glyphs are Unicode bidi class L, so a span
       holding "word-glyph + verse-marker glyph" lays them out left-to-right
       and puts the marker on the wrong side of its word. Forcing RTL fixes
       the order inside each span. Order *between* words is already correct
       here without this — these spans are flex items, so the flex container's
       rtl direction places them, not the bidi algorithm. (The Ayah list has
       no such luck: see the same fix on .ayah-text in AyahRow.) */
    unicode-bidi: bidi-override;
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
