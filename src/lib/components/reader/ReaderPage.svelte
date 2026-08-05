<script lang="ts">
  import { untrack } from 'svelte';
  import type { Ayah, ReadingScope } from '$lib/types/database';
  import ReaderView from './ReaderView.svelte';
  import PageView from './PageView.svelte';
  import AutoScrollHandle from './AutoScrollHandle.svelte';
  import ProgressIndicator from './ProgressIndicator.svelte';
  import ReaderContextBar from './ReaderContextBar.svelte';
  import ReaderZoomControl from './ReaderZoomControl.svelte';
  import ReaderJumpControl from './ReaderJumpControl.svelte';
  import TafsirPanel from '$lib/components/tafsir/TafsirPanel.svelte';
  import { uiStore, type ReadingMode } from '$lib/stores/ui.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { readingStore } from '$lib/stores/reading.svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';

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
</script>

<div class="reader-page">
  <!-- The reader and its overlays share a positioned box that the tafsir
       drawer sits beside rather than on top of: the corner controls anchor to
       the right edge of *this* element, so a drawer overlapping them would put
       the zoom buttons underneath itself. -->
  <div class="reader-main">
    {#if renderedMode === 'mushaf'}
      <PageView {ayahs} scrollToAyahId={viewTarget} />
    {:else}
      <ReaderView {ayahs} scrollToAyahId={viewTarget} />
    {/if}
    <ReaderContextBar {ayahs} />
    <ProgressIndicator />
    <AutoScrollHandle />
    <div class="control-slot">
      <ReaderZoomControl />
      <ReaderJumpControl />
    </div>
  </div>
  {#if tafsirStore.open}
    <TafsirPanel />
  {/if}
</div>

<style>
  .reader-page {
    position: relative;
    display: flex;
    height: 100%;
    overflow: hidden;
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
    right: 30px;
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
