import { uiStore } from './ui.svelte';

export const AUTO_SCROLL_SPEED_MIN = 0;
export const AUTO_SCROLL_SPEED_MAX = 100;
/** One nudge of the speed, shared by the handle's arrow keys and the shortcuts. */
export const AUTO_SCROLL_SPEED_STEP = 10;

const DEFAULT_SPEED = 40;
/** Rate at full speed and 100% reader zoom; see the scaling in `tick`. */
const MAX_PX_PER_SECOND = 40;
// Higher = snappier ramp to the target speed; frame-rate independent (exponential ease).
const EASE_RATE = 6;
/**
 * The longest frame `tick` will account for. A frame that ran long — a font
 * group registering, a page of glyphs going in — would otherwise be paid back
 * as one multi-pixel jump on the next, making the hitch look worse than it
 * was. And WebKit pauses animation frames while the window is hidden, so a
 * minimised reader would come back having "scrolled" for the whole time it
 * was away.
 */
const MAX_FRAME_SECONDS = 0.05;

class AutoScrollStore {
  speed = $state(0);
  /** The handle's on/off: a pace above zero is set. */
  active = $derived(this.speed > 0);
  /**
   * Whether the scroll loop has work to do — on from the moment a pace is set
   * until the eased speed has wound down to nothing after it's cleared, so a
   * stop glides to a halt instead of cutting out. The views key the loop on
   * this rather than on `speed`, so dragging the handle or nudging the pace
   * doesn't tear the loop down and restart it on every change.
   */
  running = $state(false);

  #lastSpeed = DEFAULT_SPEED;
  #currentPxPerSecond = 0;
  #accumulator = 0;

  toggle() {
    this.setSpeed(this.speed > 0 ? 0 : this.#lastSpeed);
  }

  /** Halt on the spot, no wind-down — for the end of the range and for follow. */
  stop() {
    this.speed = 0;
    this.running = false;
    this.#currentPxPerSecond = 0;
    this.#accumulator = 0;
  }

  faster() {
    this.setSpeed(this.speed + AUTO_SCROLL_SPEED_STEP);
  }

  /** Stops at zero, easing down through `tick` rather than cutting out. */
  slower() {
    this.setSpeed(this.speed - AUTO_SCROLL_SPEED_STEP);
  }

  setSpeed(value: number) {
    const clamped = Math.min(
      AUTO_SCROLL_SPEED_MAX,
      Math.max(AUTO_SCROLL_SPEED_MIN, Math.round(value)),
    );
    this.speed = clamped;
    if (clamped > 0) {
      this.#lastSpeed = clamped;
      this.running = true;
    }
  }

  /**
   * Advance the eased scroll speed toward the target for this frame.
   * `scrollTop` can't be trusted to keep a fractional remainder across
   * writes — WebKitGTK snaps it to the nearest integer on every set — so
   * the sub-pixel accumulator has to live here, and only whole pixels are
   * ever committed to `whole`. The leftover (`fraction`, always in [0,1))
   * is exposed so the caller can render it as a CSS transform on top of
   * the real scroll position: without that, low speeds visibly step once
   * every N frames instead of gliding.
   */
  tick(dt: number): { whole: number; fraction: number } {
    // Negative too: the loop's first frame is timed against performance.now()
    // at mount, which can land after that frame's own timestamp.
    dt = Math.min(Math.max(dt, 0), MAX_FRAME_SECONDS);

    // Scaled by reader zoom so a handle position means a *reading* pace rather
    // than a pixel rate: every line is twice as tall at 200% zoom, so a fixed
    // px/s would read half as fast for the same setting. Zoom changes ease in
    // through the same ramp below as a speed change does.
    //
    // Exact in the Ayah list, where the text size is a plain multiple of the
    // zoom. In the Mushaf view the line size is additionally capped to what
    // the column can hold (see `.text-line` in PageView), so once the column
    // has grown to fill the window the lines stop growing while this keeps
    // scaling — a high zoom in a narrow window still runs somewhat fast.
    const target = (this.speed / AUTO_SCROLL_SPEED_MAX) * MAX_PX_PER_SECOND * uiStore.readerZoom;
    const alpha = 1 - Math.exp(-EASE_RATE * dt);
    this.#currentPxPerSecond += (target - this.#currentPxPerSecond) * alpha;

    if (target === 0 && Math.abs(this.#currentPxPerSecond) < 0.05) {
      this.#currentPxPerSecond = 0;
      this.#accumulator = 0;
      this.running = false;
      return { whole: 0, fraction: 0 };
    }

    this.#accumulator += this.#currentPxPerSecond * dt;
    const whole = Math.trunc(this.#accumulator);
    this.#accumulator -= whole;
    return { whole, fraction: this.#accumulator };
  }
}

export const autoScrollStore = new AutoScrollStore();

/**
 * Run the scroll loop against a view's scroller until the returned cleanup is
 * called. Both reader views mount it from an effect keyed on `running`.
 *
 * `content` is the scroller's one child, and carries the sub-pixel remainder
 * `tick` hands back as a transform (see there). It's promoted to a compositing
 * layer of its own only while the loop runs: a standing `will-change` kept a
 * layer the height of the whole range alive with auto-scroll off, too.
 */
export function driveAutoScroll(container: HTMLElement, content: HTMLElement): () => void {
  let raf = 0;
  let last = performance.now();
  content.style.willChange = 'transform';

  function step(now: number) {
    const dt = (now - last) / 1000;
    last = now;
    const { whole, fraction } = autoScrollStore.tick(dt);
    if (whole !== 0) container.scrollTop += whole;
    content.style.transform = fraction > 0 ? `translateY(${-fraction}px)` : '';
    if (container.scrollTop + container.clientHeight >= container.scrollHeight - 1) {
      autoScrollStore.stop();
      return;
    }
    if (autoScrollStore.running) raf = requestAnimationFrame(step);
  }
  raf = requestAnimationFrame(step);

  return () => {
    cancelAnimationFrame(raf);
    content.style.transform = '';
    content.style.willChange = '';
  };
}
