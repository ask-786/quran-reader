<script lang="ts">
  import { untrack } from 'svelte';
  import { X, PanelRight, ChevronLeft, ChevronRight } from 'lucide-svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import TafsirMeta from './TafsirMeta.svelte';
  import TafsirAudioRow from '$lib/components/audio/TafsirAudioRow.svelte';
  import TafsirBody from './TafsirBody.svelte';

  /** Gap between the anchor and the card, and the arrow's own size. */
  const GAP = 10;
  const ARROW = 7;
  /** Keep the card this far from the viewport edges. */
  const MARGIN = 8;
  /** Width when the reader has not chosen one with the grip. */
  const WIDTH = 420;
  /** Narrowest the grip will make the card — the side panel's own minimum. */
  const MIN_WIDTH = 280;
  /** Below this there is no point flipping to a side — nothing would fit. Also
   *  the shortest the grip will make the card. */
  const MIN_HEIGHT = 120;
  /** Grip keyboard steps, as on the side panel's resize handle. */
  const STEP = 10;
  const BIG_STEP = 40;
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
  let width = $state(WIDTH);
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
   * The exception is a step from the card's own previous/next buttons: the
   * card stays put, as if moved, so those buttons stay under the pointer.
   */
  let moved = $state(false);
  let dragging = $state(false);
  /** The pointer's offset into the card when the drag started, so the card
   *  moves with the pointer instead of jumping its corner to it. */
  let grab: { pointerId: number; x: number; y: number; dx: number; dy: number } | null = null;
  /** The verse the card was last placed for. Not reactive, because it only
   *  exists to tell a re-aim apart from a re-run of the placement effect. */
  let placedFor: number | null = null;

  /**
   * The size being dragged with the grip, before it is saved. Null otherwise,
   * when the saved size (or the automatic one) is in force.
   */
  let liveWidth = $state<number | null>(null);
  let liveHeight = $state<number | null>(null);
  let sizing: { pointerId: number; x: number; y: number; w: number; h: number } | null = null;
  let resizing = $state(false);

  const selection = $derived(tafsirStore.selection);

  /** The height the reader chose, or null where it follows the commentary. */
  function chosenHeight(): number | null {
    return liveHeight ?? (tafsirStore.cardHeight || null);
  }

  /**
   * The width to use in a window this wide. Never wider than the window allows,
   * whatever was saved — a size chosen on a large screen must not push the card
   * off a smaller one.
   */
  function fitWidth(vw: number) {
    const room = vw - MARGIN * 2;
    const wanted = liveWidth ?? (tafsirStore.cardWidth || WIDTH);
    return Math.min(room, Math.max(MIN_WIDTH, wanted));
  }

  /** Same for height: the tallest a card can be and still be whole on screen. */
  function fitHeight(h: number, vh: number) {
    return Math.min(h, vh - MARGIN * 2);
  }

  function place() {
    if (!card) return;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    width = fitWidth(vw);

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
    const height = fitHeight(Math.max(MIN_HEIGHT, Math.min(wanted, available)), vh);

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
    const chosen = chosenHeight();
    if (chosen !== null) return chosen;
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
    // The MIN_HEIGHT floor loses to the window: in one shorter than that the
    // card shrinks rather than hanging off the bottom.
    maxHeight = Math.min(
      Math.max(MIN_HEIGHT, Math.min(wantedHeight(vh), vh - top - MARGIN)),
      vh - top - MARGIN,
    );
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
    settle(fitWidth(vw), vw, window.innerHeight);
  }

  function endDrag(e: PointerEvent) {
    if (!grab || e.pointerId !== grab.pointerId) return;
    if (head?.hasPointerCapture(e.pointerId)) head.releasePointerCapture(e.pointerId);
    grab = null;
    dragging = false;
  }

  /**
   * The corner grip. It pins the card's top-left corner and moves the
   * bottom-right one, so the card counts as moved from here on — otherwise a
   * card placed above its verse would re-place itself on every frame and grow
   * upwards, away from the pointer. Growth stops at the window's edge, not at
   * the pointer, so no part of the card can be dragged off screen.
   */
  function startResize(e: PointerEvent) {
    if (e.button !== 0 || !card) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    const rect = card.getBoundingClientRect();
    sizing = { pointerId: e.pointerId, x: e.clientX, y: e.clientY, w: rect.width, h: rect.height };
    resizing = true;
  }

  function moveResize(e: PointerEvent) {
    if (!sizing || e.pointerId !== sizing.pointerId) return;
    // Here rather than on the press, so a click or double-click on the grip
    // leaves the card beside its verse.
    moved = true;
    resizeTo(sizing.w + e.clientX - sizing.x, sizing.h + e.clientY - sizing.y);
  }

  function endResize(e: PointerEvent) {
    if (!sizing || e.pointerId !== sizing.pointerId) return;
    const grip = e.currentTarget as HTMLElement;
    if (grip.hasPointerCapture(e.pointerId)) grip.releasePointerCapture(e.pointerId);
    sizing = null;
    resizing = false;
    commitSize();
  }

  /** Arrow keys on the focused grip: left/right for width, up/down for height. */
  function nudgeSize(e: KeyboardEvent) {
    if (!card) return;
    const step = e.shiftKey ? BIG_STEP : STEP;
    const w = card.offsetWidth;
    const h = card.offsetHeight;
    const next: Record<string, [number, number]> = {
      ArrowLeft: [w - step, h],
      ArrowRight: [w + step, h],
      ArrowUp: [w, h - step],
      ArrowDown: [w, h + step],
    };
    const size = next[e.key];
    if (!size) return;
    e.preventDefault();
    moved = true;
    resizeTo(...size);
    commitSize();
  }

  function resizeTo(w: number, h: number) {
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    // The window's edge wins over the minimum, for the same reason as in
    // fitWidth: a card that can't fit whole shrinks instead of overflowing.
    liveWidth = Math.min(vw - left - MARGIN, Math.max(MIN_WIDTH, w));
    liveHeight = Math.min(vh - top - MARGIN, Math.max(MIN_HEIGHT, h));
    place();
  }

  function commitSize() {
    if (liveWidth === null || liveHeight === null) return;
    // Saved first and cleared after, in one synchronous run: the store takes
    // the new size before the live one goes, so the card never snaps back to
    // the old size for a frame.
    void tafsirStore.setCardSize(liveWidth, liveHeight);
    liveWidth = null;
    liveHeight = null;
  }

  /** Double-click the grip to go back to the automatic size. */
  function resetSize() {
    void tafsirStore.setCardSize(0, 0);
  }

  function centre(width: number, vh: number) {
    maxHeight = fitHeight(chosenHeight() ?? Math.round(vh * 0.45), vh);
    top = Math.max(MARGIN, Math.min(Math.round(vh * 0.18), vh - maxHeight - MARGIN));
    left = Math.max(MARGIN, Math.round((window.innerWidth - width) / 2));
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
      moved = selection?.stepped === true;
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
  // keyboard path is a round trip rather than a one-way door. The trigger is
  // read untracked so that stepping to another verse, which replaces the
  // selection, does not pull focus off the step button onto the card.
  $effect(() => {
    if (!card) return;
    const trigger = untrack(() => selection?.anchor ?? null);
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
    style:width="{width}px"
    style:max-height="{maxHeight}px"
    style:height={chosenHeight() !== null ? `${maxHeight}px` : null}
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
          onclick={() => tafsirStore.step(-1)}
          disabled={!tafsirStore.canStepBack}
          aria-label="Previous verse"
          title="Previous verse"
        >
          <ChevronLeft size={15} />
        </button>
        <button
          class="icon-btn"
          onclick={() => tafsirStore.step(1)}
          disabled={!tafsirStore.canStepForward}
          aria-label="Next verse"
          title="Next verse"
        >
          <ChevronRight size={15} />
        </button>
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

    <!-- Mouse, touch and keyboard, like the side panel's handle: arrows resize
         while it is focused, and a double-click goes back to automatic. -->
    <button
      type="button"
      class="resize-grip"
      class:resizing
      aria-label="Resize card, currently {width} by {Math.round(
        maxHeight,
      )} pixels. Double-click to reset."
      title="Drag to resize · double-click to reset"
      onpointerdown={startResize}
      onpointermove={moveResize}
      onpointerup={endResize}
      onpointercancel={endResize}
      onkeydown={nudgeSize}
      ondblclick={resetSize}
    ></button>
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
    /* Width and height are set inline from place(), which fits both to the
       window every time it runs — including on window resize. */
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

  .icon-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  /* Fills whatever height the grip gave the card; the header and audio row
     keep their own. */
  .body {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
  }

  .head,
  .popover > div:not(.body) {
    flex-shrink: 0;
  }

  /* Two diagonal strokes in the corner, the usual sign for a resize grip. */
  .resize-grip {
    position: absolute;
    right: 0;
    bottom: 0;
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    border-bottom-right-radius: var(--radius);
    background: linear-gradient(
      135deg,
      transparent 45%,
      var(--color-text-muted) 45% 52%,
      transparent 52% 68%,
      var(--color-text-muted) 68% 75%,
      transparent 75%
    );
    opacity: 0.45;
    cursor: nwse-resize;
    touch-action: none;
  }

  .resize-grip:hover,
  .resize-grip:focus-visible,
  .resize-grip.resizing {
    opacity: 0.9;
    outline: none;
  }
</style>
