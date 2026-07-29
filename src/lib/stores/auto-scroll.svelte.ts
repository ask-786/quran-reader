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

class AutoScrollStore {
  speed = $state(0);

  #lastSpeed = DEFAULT_SPEED;
  #currentPxPerSecond = 0;
  #accumulator = 0;

  get active() {
    return this.speed > 0;
  }

  toggle() {
    this.setSpeed(this.speed > 0 ? 0 : this.#lastSpeed);
  }

  stop() {
    this.speed = 0;
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
    if (clamped > 0) this.#lastSpeed = clamped;
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
      return { whole: 0, fraction: 0 };
    }

    this.#accumulator += this.#currentPxPerSecond * dt;
    const whole = Math.trunc(this.#accumulator);
    this.#accumulator -= whole;
    return { whole, fraction: this.#accumulator };
  }
}

export const autoScrollStore = new AutoScrollStore();
