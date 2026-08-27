<script lang="ts">
  import { X, PanelRight } from 'lucide-svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import TafsirMeta from './TafsirMeta.svelte';
  import TafsirBody from './TafsirBody.svelte';

  /** Gap between the anchor and the card, and the arrow's own size. */
  const GAP = 10;
  const ARROW = 7;
  /** Keep the card this far from the viewport edges. */
  const MARGIN = 8;
  const WIDTH = 420;
  /** Below this there is no point flipping to a side — nothing would fit. */
  const MIN_HEIGHT = 120;

  let card = $state<HTMLDivElement>();
  let head = $state<HTMLElement>();
  let body = $state<HTMLDivElement>();

  /**
   * Last computed placement. Kept in state rather than recomputed during render
   * because it depends on the card's own measured height, which only exists
   * after it has rendered once.
   */
  let top = $state(0);
  let left = $state(0);
  let maxHeight = $state(0);
  let arrowTop = $state(false);
  let arrowLeft = $state(0);
  /** Hidden until the first placement, so it never flashes at 0,0. */
  let placed = $state(false);

  const selection = $derived(tafsirStore.selection);

  function place() {
    if (!card) return;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const width = Math.min(WIDTH, vw - MARGIN * 2);

    const anchor = selection?.anchor ?? null;
    // Freeze rather than chase. Three ways an anchor stops being usable, and
    // all of them have to be caught here or the card walks off the screen:
    //
    //  - it leaves the DOM, because both views drop content outside their
    //    render window (a detached element reports a zero rect at the origin);
    //  - it reports a zero rect for the same reason;
    //  - it is still attached but has scrolled out of the viewport, at which
    //    point following it means placing the card at a negative offset and
    //    the reader simply sees it disappear.
    //
    // The last one is what actually happens when you scroll a long Surah with
    // a popover open, and it is the one this originally got wrong.
    const rect = anchor?.isConnected ? anchor.getBoundingClientRect() : null;
    const usable =
      rect !== null && (rect.width > 0 || rect.height > 0) && rect.bottom > 0 && rect.top < vh;
    if (!usable) {
      if (!placed) centre(width, vh);
      return;
    }

    const spaceBelow = vh - rect.bottom - GAP - MARGIN;
    const spaceAbove = rect.top - GAP - MARGIN;
    // Measure the *content*, not the card. The card is already constrained by
    // the max-height this function set last time, so asking it how tall it
    // wants to be is circular — it answers with the answer it was given, and
    // the popover never grows past its first (zero) placement. The body is the
    // scroll container, so its scrollHeight is the full text either way.
    const natural = (head?.offsetHeight ?? 0) + (body?.scrollHeight ?? 0) + 2;
    const wanted = Math.min(natural, Math.round(vh * 0.45));

    let below = spaceBelow >= Math.min(wanted, MIN_HEIGHT);
    // Prefer the side that actually has room; only when neither does is the
    // larger of the two picked and the card shrunk into it. Either way it is
    // placed *beside* the anchor rect, never over it — which is the point.
    if (below && spaceBelow < wanted && spaceAbove > spaceBelow) below = false;

    const available = below ? spaceBelow : spaceAbove;
    const height = Math.max(MIN_HEIGHT, Math.min(wanted, available));

    maxHeight = height;
    // Clamped into the viewport as well as chosen by side: with the anchor
    // straddling an edge, the arithmetic above can still land the card
    // partly outside, and a card you cannot read is no better than no card.
    const wantedTop = below ? rect.bottom + GAP : rect.top - GAP - height;
    top = Math.min(Math.max(wantedTop, MARGIN), Math.max(MARGIN, vh - height - MARGIN));
    arrowTop = below;

    const centreX = rect.left + rect.width / 2;
    left = Math.min(Math.max(centreX - width / 2, MARGIN), vw - width - MARGIN);
    // Clamped separately from the card so it keeps pointing at the anchor even
    // when the card itself has been pushed off centre by the viewport edge.
    arrowLeft = Math.min(Math.max(centreX - left, ARROW * 2), width - ARROW * 2);
    placed = true;
  }

  function centre(width: number, vh: number) {
    maxHeight = Math.round(vh * 0.45);
    top = Math.round(vh * 0.18);
    left = Math.round((window.innerWidth - width) / 2);
    arrowTop = true;
    arrowLeft = -100; // off-card: there is nothing to point at
    placed = true;
  }

  // Place once the card exists, and again whenever the selection changes. The
  // body's content arrives asynchronously, so this also re-runs on the entry.
  $effect(() => {
    void selection;
    void tafsirStore.entry;
    void tafsirStore.loading;
    if (!card) return;
    place();
  });

  $effect(() => {
    if (!card) return;
    let frame = 0;
    const schedule = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(place);
    };
    // Capture, because the scroll that matters happens inside the reader's own
    // scroll container and never reaches window in the bubble phase.
    window.addEventListener('scroll', schedule, true);
    window.addEventListener('resize', schedule);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener('scroll', schedule, true);
      window.removeEventListener('resize', schedule);
    };
  });

  // Focus moves in on open and back to whatever opened it on close, so the
  // keyboard path is a round trip rather than a one-way door.
  $effect(() => {
    if (!card) return;
    const trigger = selection?.anchor ?? null;
    card.focus({ preventScroll: true });
    return () => {
      if (trigger?.isConnected && typeof trigger.focus === 'function') {
        trigger.focus({ preventScroll: true });
      }
    };
  });

  /**
   * Move this element to the body. The reader is inside overflow/transform
   * ancestors, and a fixed-position element inside a transformed ancestor is
   * positioned against *that* ancestor rather than the viewport — the reader
   * sets `transform` on its content while auto-scrolling, so this is not
   * hypothetical.
   */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }
</script>

<div class="portal" use:portal>
  <div
    bind:this={card}
    class="popover"
    class:placed
    data-tafsir-surface
    role="dialog"
    aria-label="Tafsir"
    tabindex="-1"
    style:top="{top}px"
    style:left="{left}px"
    style:max-height="{maxHeight}px"
  >
    <span class="arrow" class:above={!arrowTop} style:left="{arrowLeft}px"></span>

    <header class="head" bind:this={head}>
      <TafsirMeta compact />
      <div class="actions">
        <button
          class="icon-btn"
          onclick={() => tafsirStore.setView('panel')}
          aria-label="Open in side panel"
          title="Open in side panel"
        >
          <PanelRight size={15} />
        </button>
        <button
          class="icon-btn"
          onclick={() => tafsirStore.closePopover()}
          aria-label="Close tafsir"
        >
          <X size={15} />
        </button>
      </div>
    </header>

    <div class="body" bind:this={body}>
      <TafsirBody />
    </div>
  </div>
</div>

<style>
  /* The portal wrapper itself must not take part in layout at all — it is only
     a handle for the action that moves the card to the body. */
  .portal {
    display: contents;
  }

  .popover {
    position: fixed;
    z-index: 60;
    display: flex;
    flex-direction: column;
    width: min(420px, calc(100vw - 16px));
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
    /* Hidden until placed, so the first paint never lands at the origin. */
    visibility: hidden;
    outline: none;
  }

  .popover.placed {
    visibility: visible;
  }

  .arrow {
    position: absolute;
    width: 12px;
    height: 12px;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border);
    border-top: 1px solid var(--color-border);
    transform: translateX(-50%) rotate(45deg);
    top: -7px;
  }

  /* Pointing down, at an anchor below the card. */
  .arrow.above {
    top: auto;
    bottom: -7px;
    border-left: none;
    border-top: none;
    border-right: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 9px 8px 9px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .actions {
    display: flex;
    flex-shrink: 0;
    gap: 2px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .body {
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
  }
</style>
