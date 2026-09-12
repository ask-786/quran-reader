<script lang="ts">
  import { X, PanelRight } from 'lucide-svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import TafsirMeta from './TafsirMeta.svelte';
  import TafsirAudioRow from '$lib/components/audio/TafsirAudioRow.svelte';
  import TafsirBody from './TafsirBody.svelte';

  /** Gap between the anchor and the card, and the arrow's own size. */
  const GAP = 10;
  const ARROW = 7;
  /** Keep the card this far from the viewport edges. */
  const MARGIN = 8;
  const WIDTH = 420;
  /** Below this there is no point flipping to a side — nothing would fit. */
  const MIN_HEIGHT = 120;
  /**
   * What a press on the header must not turn into a drag: its own controls,
   * and the edition menu, which opens inside the header. Dragging from those
   * would swallow the click they exist for.
   */
  const NOT_A_HANDLE = 'button, a[href], input, select, [role="listbox"], [role="option"]';
  /** How far the pointer has to travel before a press on the header counts as
   *  a drag. A hand clicking the header still moves a pixel or two, and that
   *  should not pull the card off its verse. */
  const DRAG_SLOP = 4;

  let card = $state<HTMLDivElement>();
  let head = $state<HTMLElement>();
  let audio = $state<HTMLDivElement>();
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

  /**
   * Set once the reader has dragged the card by its header, and from then on
   * the card stays where they put it instead of beside its verse.
   *
   * Dragging is for one situation: the card sits beside the verse it opened
   * for, and while that verse is being recited the reader wants to follow the
   * text, which may be the lines the card covers. Once the card has been moved
   * aside, putting it back on every scroll or entry load would undo the
   * move. So a moved card only keeps itself inside the viewport, and it loses
   * its arrow, which would otherwise point at wherever it was dropped.
   *
   * Reset when the card is aimed at a different verse. A card opened for a new
   * verse starts beside that verse, like any card that has just been opened.
   */
  let moved = $state(false);
  let dragging = $state(false);
  /** The pointer's offset into the card when the drag started, so the card
   *  moves with the pointer instead of jumping its corner to it. */
  let grab: { pointerId: number; x: number; y: number; dx: number; dy: number } | null = null;
  /** The verse the card was last placed for. Not reactive, because it only
   *  exists to tell a re-aim apart from a re-run of the placement effect. */
  let placedFor: number | null = null;

  const selection = $derived(tafsirStore.selection);

  function place() {
    if (!card) return;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const width = Math.min(WIDTH, vw - MARGIN * 2);

    if (moved) {
      settle(width, vw, vh);
      return;
    }

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
    //
    // The cap is on the commentary, not on the card. Header and audio strip are
    // chrome, and counting them inside the 45% is what made adding the strip
    // read as the tafsir shrinking: the card kept its height and the text paid
    // for the row. Adding chrome on top instead leaves the commentary exactly
    // the room it had before the strip existed.
    const wanted = wantedHeight(vh);

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

  function wantedHeight(vh: number) {
    const chrome = (head?.offsetHeight ?? 0) + (audio?.offsetHeight ?? 0) + 2;
    const text = body?.scrollHeight ?? 0;
    return chrome + Math.min(text, Math.round(vh * 0.45));
  }

  /**
   * Placement for a card the reader has moved: leave it where it is, and only
   * pull it back inside the viewport, since a window resize can leave it
   * partly off screen.
   *
   * The card may be dropped low enough that its full height would run off the
   * bottom. Its height gives way rather than its position, down to
   * MIN_HEIGHT, because the reader chose that position and the commentary
   * scrolls anyway. Dragging it back up gives the height back, since this
   * runs on every move.
   */
  function settle(width: number, vw: number, vh: number) {
    left = Math.min(Math.max(left, MARGIN), Math.max(MARGIN, vw - width - MARGIN));
    top = Math.min(Math.max(top, MARGIN), Math.max(MARGIN, vh - MIN_HEIGHT - MARGIN));
    maxHeight = Math.max(MIN_HEIGHT, Math.min(wantedHeight(vh), vh - top - MARGIN));
  }

  function startDrag(e: PointerEvent) {
    if (e.button !== 0 || !head) return;
    if (e.target instanceof Element && e.target.closest(NOT_A_HANDLE)) return;
    // Otherwise the press also starts a text selection across the header, and
    // the reader's text under it once the pointer moves off the card.
    e.preventDefault();
    head.setPointerCapture(e.pointerId);
    grab = {
      pointerId: e.pointerId,
      x: e.clientX,
      y: e.clientY,
      dx: e.clientX - left,
      dy: e.clientY - top,
    };
    dragging = true;
  }

  function moveDrag(e: PointerEvent) {
    if (!grab || e.pointerId !== grab.pointerId) return;
    // Set once the pointer has travelled, not on the press. A click on the
    // header leaves the card attached to its verse.
    if (!moved) {
      if (Math.hypot(e.clientX - grab.x, e.clientY - grab.y) < DRAG_SLOP) return;
      moved = true;
    }
    left = e.clientX - grab.dx;
    top = e.clientY - grab.dy;
    const vw = window.innerWidth;
    settle(Math.min(WIDTH, vw - MARGIN * 2), vw, window.innerHeight);
  }

  function endDrag(e: PointerEvent) {
    if (!grab || e.pointerId !== grab.pointerId) return;
    if (head?.hasPointerCapture(e.pointerId)) head.releasePointerCapture(e.pointerId);
    grab = null;
    dragging = false;
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
    // Compared by verse, not by selection object. Opening the same verse again
    // makes a new object, and that should not undo a move.
    const ayahId = selection?.ayahId ?? null;
    if (ayahId !== placedFor) {
      placedFor = ayahId;
      moved = false;
    }
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

    // The audio strip is the one part of the card that changes height on its
    // own — a permission line or an error appears under the controls — and the
    // card's budget is computed from it, so a silent change would take the
    // difference out of the commentary.
    let sizes: ResizeObserver | undefined;
    if (audio) {
      sizes = new ResizeObserver(schedule);
      sizes.observe(audio);
    }

    return () => {
      cancelAnimationFrame(frame);
      sizes?.disconnect();
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
    aria-label="Verse"
    tabindex="-1"
    style:top="{top}px"
    style:left="{left}px"
    style:max-height="{maxHeight}px"
  >
    {#if !moved}
      <span class="arrow" class:above={!arrowTop} style:left="{arrowLeft}px"></span>
    {/if}

    <!-- The header is the drag handle. Mouse and touch only: the card is a
         dialog that keyboard users reach and leave through focus, and its
         position does not decide what they can read. -->
    <header
      class="head"
      class:dragging
      bind:this={head}
      onpointerdown={startDrag}
      onpointermove={moveDrag}
      onpointerup={endDrag}
      onpointercancel={endDrag}
    >
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
        <button class="icon-btn" onclick={() => tafsirStore.closePopover()} aria-label="Close">
          <X size={15} />
        </button>
      </div>
    </header>

    <!-- Wrapped so `place()` can measure it: the card's height budget has to
         know this row is here, or it comes out of the commentary. -->
    <div bind:this={audio}>
      {#if selection}
        <TafsirAudioRow ayahId={selection.ayahId} />
      {/if}
    </div>

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
    /* The drag handle. `grab` shows that the header can be picked up, and
       `grabbing` shows that it has been. The controls inside keep their own
       pointer cursor, which also shows that pressing them will not drag. */
    cursor: grab;
    /* Without this a touch drag scrolls the page instead of moving the
       card. */
    touch-action: none;
  }

  .head.dragging {
    cursor: grabbing;
    user-select: none;
  }

  .actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
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
