<script lang="ts">
  import { untrack } from 'svelte';
  import type { Ayah } from '$lib/types/database';
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
  import { tafsirStore } from '$lib/stores/tafsir.svelte';

  let {
    ayahs,
    scrollTarget,
  }: {
    ayahs: Ayah[];
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
      if (ayahsChanged) readerPosition.reset();
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
