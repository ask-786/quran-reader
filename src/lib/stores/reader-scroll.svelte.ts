/**
 * Scroll commands for whichever reader view is mounted.
 *
 * The keyboard handler lives in the root layout and the jump buttons live
 * beside the zoom control, but the element that actually scrolls belongs to
 * ReaderView or PageView. Rather than plumbing callbacks down through
 * ReaderPage, the mounted view registers its scroll container here and both
 * callers drive it through this store. Nothing is registered on the home
 * route, so every command is a no-op there instead of an error.
 */

/**
 * How far past a page's start counts as "into" that page rather than at it,
 * for the backwards jump below. A few pixels of rounding shouldn't make
 * `ArrowUp` land on the page you are already looking at.
 */
const AT_PAGE_START_EPSILON = 40;

class ReaderScrollStore {
  #container: HTMLElement | null = null;
  /** Selector for the elements the registered view tags with `data-page`. */
  #anchorSelector = '';

  /**
   * Called by the mounted view once its container exists. Returns the
   * teardown, so callers can hand it straight back from an `$effect`.
   */
  register(container: HTMLElement, anchorSelector: string) {
    this.#container = container;
    this.#anchorSelector = anchorSelector;
    return () => {
      // Only clear if we're still the registered view: on a Mushaf/list toggle
      // the incoming view registers before the outgoing one tears down, and
      // clearing unconditionally would leave nothing registered at all.
      if (this.#container === container) {
        this.#container = null;
        this.#anchorSelector = '';
      }
    };
  }

  /**
   * The first element of each Mushaf page currently in the document, in page
   * order. Both views keep every page's box in the DOM whether or not its
   * glyphs are rendered (see the windowing in ReaderView/PageView), so this
   * covers the whole open range, not just what's on screen.
   */
  #pageAnchors(): { page: number; el: HTMLElement }[] {
    const container = this.#container;
    if (!container) return [];
    const anchors: { page: number; el: HTMLElement }[] = [];
    // Both views lay their pages out in ascending order and never interleave
    // them, so document order is page order and the first element carrying a
    // page number is that page's start — no sorting or deduplication needed
    // beyond skipping the run of elements repeating the current page.
    for (const el of container.querySelectorAll<HTMLElement>(this.#anchorSelector)) {
      const page = Number(el.dataset.page);
      if (!Number.isFinite(page) || page === anchors[anchors.length - 1]?.page) continue;
      anchors.push({ page, el });
    }
    return anchors;
  }

  #scrollTo(top: number) {
    const container = this.#container;
    if (!container) return;
    // Instant, not smoothed: auto-scroll writes scrollTop every frame, and a
    // smooth scroll running alongside it fights those writes and stalls
    // halfway. See the scroll-behavior note in ReaderView's styles.
    container.scrollTo({ top: Math.max(0, top), behavior: 'auto' });
  }

  /**
   * Move one Mushaf page boundary within the open range. Past either end of
   * that range this does nothing: stepping off the last page of a Surah into
   * the next one is what `n`/`p` are for, and on the single-page route the
   * layout routes the same keys to page navigation instead.
   */
  jumpPage(direction: 1 | -1) {
    const container = this.#container;
    if (!container) return;
    const anchors = this.#pageAnchors();
    if (anchors.length === 0) return;

    const containerTop = container.getBoundingClientRect().top;
    const offsets = anchors.map((a) => a.el.getBoundingClientRect().top - containerTop);

    // The page being read is the last one whose start is at or above the top
    // of the viewport.
    let currentIndex = 0;
    for (let i = 0; i < offsets.length; i++) {
      if (offsets[i] <= AT_PAGE_START_EPSILON) currentIndex = i;
      else break;
    }

    let targetIndex: number;
    if (direction === 1) {
      targetIndex = currentIndex + 1;
    } else {
      // Scrolled partway into a page, `ArrowUp` returns to that page's own
      // start first — the same "previous track" behaviour a media player has,
      // and the alternative (always skipping to the page before) would step
      // over a boundary the reader never got to see from the top.
      targetIndex =
        offsets[currentIndex] < -AT_PAGE_START_EPSILON ? currentIndex : currentIndex - 1;
    }
    if (targetIndex < 0 || targetIndex >= anchors.length) return;

    this.#scrollTo(container.scrollTop + offsets[targetIndex]);
  }

  scrollToTop() {
    this.#scrollTo(0);
  }

  scrollToBottom() {
    const container = this.#container;
    if (!container) return;
    this.#scrollTo(container.scrollHeight);
  }
}

export const readerScroll = new ReaderScrollStore();
