<script lang="ts">
  import { untrack } from 'svelte';
  import type { Ayah, RangeFocus, ReadingScope } from '$lib/types/database';
  import ReaderView from './ReaderView.svelte';
  import PageView from './PageView.svelte';
  import AutoScrollHandle from './AutoScrollHandle.svelte';
  import ProgressIndicator from './ProgressIndicator.svelte';
  import ReaderContextBar from './ReaderContextBar.svelte';
  import ReaderZoomControl from './ReaderZoomControl.svelte';
  import ReaderJumpControl from './ReaderJumpControl.svelte';
  import ListenPanel from '$lib/components/audio/ListenPanel.svelte';
  import TafsirPanel from '$lib/components/tafsir/TafsirPanel.svelte';
  import TafsirPopover from '$lib/components/tafsir/TafsirPopover.svelte';
  import { uiStore, type ReadingMode } from '$lib/stores/ui.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { readingStore } from '$lib/stores/reading.svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import { autoScrollStore } from '$lib/stores/auto-scroll.svelte';
  import { scrollToRecitedAyah } from '$lib/utils/follow-scroll';

  let {
    ayahs,
    scope,
    scopeId,
    scrollTarget,
  }: {
    ayahs: Ayah[];
    /** The range these Ayahs are, for recording where in it the reader is. */
    scope: ReadingScope;
    scopeId: number;
    scrollTarget?: number;
  } = $props();

  // Where the mounted view should scroll to. Normally that's whatever the route
  // asked for, but toggling reading mode tears one view down and builds the
  // other, so on that trigger alone we hand the incoming view the Ayah the
  // outgoing one was showing.
  let viewTarget = $state<number | undefined>();

  // Which view the template actually shows. Deliberately *not* read straight
  // from uiStore: Svelte updates the template before running user effects, so
  // an `{#if uiStore.readingMode}` would mount the incoming view a beat before
  // the effect below hands it the handoff Ayah — it would come up pointed at
  // the *previous* target. That used to be masked by the page load being slow
  // enough that the correction always won, but the data is cached now and a
  // warm toggle renders immediately. Writing this after `viewTarget` in the
  // same effect run means the swap and the target land in one update, so the
  // incoming view is correct on its very first render.
  let renderedMode = $state<ReadingMode>(uiStore.readingMode);

  // Plain (non-reactive) last-seen values: the point is to tell *which* input
  // changed. Deliberately one flat $effect rather than a $derived reading the
  // position through untrack() — a derived evaluates during render, before the
  // effect that refreshes the tracked position has run, so a stale position
  // wins the race and swallows genuinely new targets like `?ayah=` deep links.
  let lastAyahs: Ayah[] | undefined;
  let lastScrollTarget: number | undefined;
  let lastReadingMode: ReadingMode | undefined;

  $effect(() => {
    const currentAyahs = ayahs;
    const target = scrollTarget;
    const mode = uiStore.readingMode;

    const ayahsChanged = currentAyahs !== lastAyahs;
    const targetChanged = target !== lastScrollTarget;
    const modeChanged = mode !== lastReadingMode;

    lastAyahs = currentAyahs;
    lastScrollTarget = target;
    lastReadingMode = mode;

    if (ayahsChanged || targetChanged) {
      // A route change or an explicit jump (Go To, deep link, resume) always
      // wins over wherever the reader happened to be sitting.
      if (ayahsChanged) {
        // Before the position is thrown away: the last few seconds of reading
        // in the range being left are still only scheduled, and the pending
        // record carries its own scope, so this stores them against the range
        // they belong to rather than the one being opened.
        readingStore.flush();
        readerPosition.reset();
      }
      viewTarget = target;
    } else if (modeChanged) {
      // untrack: the position updates on every scroll, and re-running this on
      // each of those would re-fire the scroll and pin the view in place. The
      // previous target is read the same way, so this effect never depends on
      // the very state it writes.
      viewTarget = untrack(() => readerPosition.ayahId ?? viewTarget) ?? target;
    }

    // Last, so the swap below never renders ahead of the target above.
    renderedMode = mode;
  });

  /**
   * Persist where the reader is. Both views publish their centred Ayah to
   * `readerPosition`; recording it here rather than inside each of them is what
   * gives the Mushaf page view a remembered position at all — it never had one,
   * despite being the view the app opens in.
   *
   * Declared after the effect above so that effect's `readerPosition.reset()`
   * lands first on a route change: effects in a component run in the order they
   * were created, so this one never sees the outgoing range's Ayah paired with
   * the incoming range's scope.
   */
  $effect(() => {
    const id = readerPosition.ayahId;
    const currentScope = scope;
    const currentScopeId = scopeId;
    if (id === null || ayahs.length === 0) return;
    // Every range the reader routes cover — a Surah, a Juz, a Hizb, a Mushaf
    // page — is a contiguous run of global Ayah ids, so bounds are enough to
    // reject a position published by the view that is on its way out.
    if (id < ayahs[0].id || id > ayahs[ayahs.length - 1].id) return;
    readingStore.note(currentScope, currentScopeId, id);
  });

  // Leaving the reader entirely (focus-mode remount aside, only closing the
  // window does this) still owes the range its pending position.
  $effect(() => () => readingStore.flush());

  /**
   * The player's queue is whatever range is open — unless the listen panel is
   * up, in which case the reader stops driving it.
   *
   * That exception is the point of the panel. Someone listening to Al-Kahf
   * while turning to check a verse in another Surah has not asked for the
   * recitation to stop, and the queue following the route would do exactly
   * that. The panel owns the queue while it is open; the reader owns it the
   * rest of the time, which is what lets a verse played from the tafsir card
   * know where it sits.
   */
  $effect(() => {
    if (uiStore.listenOpen) return;
    playbackStore.setQueue(ayahs);
  });

  /**
   * Follow the recitation.
   *
   * Deliberately not routed through `viewTarget`: that prop tears the mounted
   * view's observers down and rebuilds them, which is right for a jump and far
   * too much for the verse-by-verse nudge this is. A direct scroll leaves the
   * view's own machinery alone.
   */
  $effect(() => {
    const id = playbackStore.currentAyahId;
    if (id === null || !playbackStore.shouldFollow) return;
    if (!ayahs.some((a) => a.id === id)) return;
    scrollToRecitedAyah(id);
  });

  /**
   * A verse played from a tafsir card belongs to that card. When the last
   * commentary surface closes — Escape, the close button, a click away, or a
   * switch back to popover mode — the recitation and the mark on the verse go
   * with it. Anything else leaves a highlighted āya reciting itself with
   * nothing on screen to explain why or to stop it.
   *
   * `anyOpen` covers both surfaces, so switching between them inside one update
   * never reads as a close. A range from the listen panel is untouched: it has
   * its own surface and its own stop.
   */
  $effect(() => {
    if (!tafsirStore.anyOpen) playbackStore.stopIfSingle();
  });

  /**
   * Auto-scroll and follow cannot both own the scroll position — with both on,
   * the reader is dragged by two things at once and neither lands anywhere.
   * Playback wins, since it is the one with a fixed pace of its own.
   *
   * Only on the transition into playing, so a reader who deliberately starts
   * auto-scroll mid-recitation keeps it.
   */
  let wasPlaying = false;
  $effect(() => {
    const playing = playbackStore.playing;
    if (playing && !wasPlaying && playbackStore.shouldFollow) {
      autoScrollStore.stop();
    }
    wasPlaying = playing;
  });

  /**
   * What Mushaf view does with the parts of a page outside the Ayahs this route
   * opened. A printed page is shared between Surahs, so opening Al-Mulk hands
   * you the last lines of the Surah before it and, at the far end, the opening
   * of the one after.
   *
   * Always `all` for the Mushaf page route, whatever the setting says, and not
   * as an oversight: `ayah.page` records where an Ayah *begins*, so the page
   * route's Ayah list leaves out an Ayah that started on the page before and
   * spills onto this one. Dimming or trimming to that list would grey out or
   * blank the top of the very page the reader asked to see. The other three
   * routes are ranges that may be a fraction of a page, which is the case this
   * is for.
   */
  const rangeFocus = $derived<RangeFocus>(
    scope === 'page' ? 'all' : settingsStore.current.range_focus,
  );

  /** Whether the side panel is on screen — it and the inset it opens must be
   *  the same condition, or the controls step aside for a panel that isn't
   *  there. */
  const panelOpen = $derived(tafsirStore.view === 'panel' && tafsirStore.panelOpen);
</script>

<!-- --tafsir-inset is how wide the panel is covering the right edge right now,
     0px when it is closed. The panel reads it for its own width and the corner
     controls subtract it from theirs, so one number keeps the two in step —
     including mid-drag, which is why the live width sits in the store. -->
<div
  class="reader-page"
  class:tafsir-open={panelOpen}
  style:--tafsir-width="{tafsirStore.liveWidth}px"
>
  <!-- The reader and its overlays share a positioned box. The tafsir drawer
       covers the right of it rather than shrinking it, so the reading measure
       never changes; the corner controls anchor to the right edge of *this*
       element and move inward by --tafsir-inset to stay out from under it. -->
  <div class="reader-main">
    {#if renderedMode === 'mushaf'}
      <PageView {ayahs} scrollToAyahId={viewTarget} {rangeFocus} />
    {:else}
      <ReaderView {ayahs} scrollToAyahId={viewTarget} />
    {/if}
    <ReaderContextBar {ayahs} />
    <ProgressIndicator />
    <AutoScrollHandle />
    {#if uiStore.listenOpen}
      <ListenPanel />
    {/if}
    <div class="control-slot">
      <ReaderZoomControl />
      <ReaderJumpControl />
    </div>
  </div>
  {#if panelOpen}
    <TafsirPanel />
  {/if}
</div>
<!-- Outside `.reader-page` on purpose, and moved to the body by its own portal
     once mounted: the popover must not be able to take part in the reader's
     layout, which is the whole reason it exists. -->
{#if tafsirStore.view === 'popover' && tafsirStore.selection}
  <TafsirPopover />
{/if}

<style>
  .reader-page {
    position: relative;
    display: flex;
    height: 100%;
    overflow: hidden;

    /* Closed: nothing is covered, so nothing steps aside. */
    --tafsir-inset: 0px;
  }

  .reader-page.tafsir-open {
    --tafsir-inset: var(--tafsir-width, 0px);
  }

  /* The one place the panel stops honouring the stored width: below the
     tablet breakpoint it takes the window instead. Overriding the inset here
     rather than the panel's width is what keeps the corner controls agreeing
     with it — they are outside the panel and cannot see a rule scoped to it.
     (They end up off-screen at 100%, which is correct: there is no reader
     left to control.) */
  @media (max-width: 900px) {
    .reader-page.tafsir-open {
      --tafsir-inset: min(100%, 480px);
    }
  }

  .reader-main {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* The slot owns the width so the zoom and jump controls always match: just
     wide enough for the widest zoom label ("200%") and nothing more. */
  .control-slot {
    position: absolute;
    top: 16px;
    right: calc(30px + var(--tafsir-inset));
    z-index: 6;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    width: 34px;
    gap: 8px;
  }

  @media (max-width: 480px) {
    .control-slot {
      display: none;
    }
  }
</style>
