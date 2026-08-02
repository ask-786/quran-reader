<script lang="ts">
  /**
   * The printed Mushaf carries the Surah and Juz in a header band on every
   * page — a running head, so you always know where you are without looking
   * anything up. A band per page doesn't survive the port to a continuous
   * scroll (page 62's footer and page 63's header would stack into two rows of
   * chrome at all 604 boundaries), so the same job is done once, here, by a
   * bar that floats over the top of the reader and stays out of the way:
   * hidden while you read, revealed when the answer changes or when you look
   * for it.
   */
  import type { Ayah } from '$lib/types/database';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { readerScroll } from '$lib/stores/reader-scroll.svelte';

  let { ayahs }: { ayahs: Ayah[] } = $props();

  /** How long a reveal announcing a new Surah or Juz stays up on its own. */
  const ANNOUNCE_MS = 2600;
  /** Upward scroll that counts as "where am I?" rather than a nudge. */
  const SHOW_ON_UP_PX = 40;
  /** Downward scroll that counts as settling back into reading. */
  const HIDE_ON_DOWN_PX = 80;
  /**
   * A jump, not reading: Go To, a page jump, or the re-centring the views do
   * on a zoom or reading-mode change. Those say nothing about which way the
   * reader is looking, so they reset the run rather than counting towards it.
   */
  const JUMP_PX = 400;

  // Both views publish the Ayah under the middle of the viewport; before the
  // first observer callback lands there's nothing to read but the range's own
  // start, which is where the view is about to be anyway.
  const current = $derived(ayahs.find((a) => a.id === readerPosition.ayahId) ?? ayahs[0]);
  const surah = $derived(current ? surahsStore.get(current.surah_id) : undefined);

  let visible = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function reveal(transient: boolean) {
    clearTimeout(timer);
    visible = true;
    // A reveal the reader asked for by scrolling up stays until they scroll
    // back down; one the bar volunteered retreats on its own.
    if (transient) timer = setTimeout(() => (visible = false), ANNOUNCE_MS);
  }

  function conceal() {
    clearTimeout(timer);
    visible = false;
  }

  // Announce on entering a new Surah or Juz. Deliberately reads only the two
  // values it announces — `visible` is written here and must not be tracked,
  // or every reveal would re-trigger the effect that caused it.
  $effect(() => {
    void current?.surah_id;
    void current?.juz;
    reveal(true);
  });

  $effect(() => {
    const el = readerScroll.container;
    if (!el) return;

    let lastTop = el.scrollTop;
    let up = 0;
    let down = 0;

    function onScroll() {
      const top = el!.scrollTop;
      const delta = top - lastTop;
      lastTop = top;
      if (delta === 0) return;
      if (Math.abs(delta) > JUMP_PX) {
        up = 0;
        down = 0;
        return;
      }
      if (delta < 0) {
        down = 0;
        up -= delta;
        if (up > SHOW_ON_UP_PX) reveal(false);
      } else {
        up = 0;
        down += delta;
        if (down > HIDE_ON_DOWN_PX) conceal();
      }
    }

    el.addEventListener('scroll', onScroll, { passive: true });
    return () => el.removeEventListener('scroll', onScroll);
  });

  $effect(() => () => clearTimeout(timer));
</script>

{#if surah && current}
  <div class="context-bar" class:visible aria-hidden={!visible}>
    <span>{surah.transliteration}</span>
    <span class="sep">·</span>
    <span>Juz {current.juz}</span>
  </div>
{/if}

<style>
  .context-bar {
    position: absolute;
    top: 10px;
    left: 50%;
    z-index: 7;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 11px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    background: color-mix(in srgb, var(--color-bg-elevated) 85%, transparent);
    backdrop-filter: blur(8px);
    color: var(--color-text-muted);
    font-size: 11px;
    letter-spacing: 0.03em;
    white-space: nowrap;
    /* Never in the way of a click or a text selection — the bar is a readout,
       and it sits over the top line of the reading column. */
    pointer-events: none;
    opacity: 0;
    transform: translate(-50%, -14px);
    transition:
      opacity 160ms ease,
      transform 160ms ease;
  }

  .context-bar.visible {
    opacity: 1;
    transform: translate(-50%, 0);
  }

  .sep {
    color: var(--color-text-faint);
  }

  @media (prefers-reduced-motion: reduce) {
    .context-bar {
      transform: translate(-50%, 0);
      transition: opacity 160ms ease;
    }
  }
</style>
