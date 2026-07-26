export const AUTO_SCROLL_SPEED_MIN = 0;
export const AUTO_SCROLL_SPEED_MAX = 100;

const DEFAULT_SPEED = 40;
const MAX_PX_PER_SECOND = 150;
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

  setSpeed(value: number) {
    const clamped = Math.min(
      AUTO_SCROLL_SPEED_MAX,
      Math.max(AUTO_SCROLL_SPEED_MIN, Math.round(value)),
    );
    this.speed = clamped;
    if (clamped > 0) this.#lastSpeed = clamped;
  }

  /**
   * Advance the eased scroll speed toward the target for this frame and
   * return the whole pixels to apply. The fractional remainder is kept in
   * an accumulator so low speeds still scroll smoothly instead of getting
   * rounded away every frame.
   */
  tick(dt: number): number {
    const target = (this.speed / AUTO_SCROLL_SPEED_MAX) * MAX_PX_PER_SECOND;
    const alpha = 1 - Math.exp(-EASE_RATE * dt);
    this.#currentPxPerSecond += (target - this.#currentPxPerSecond) * alpha;

    if (target === 0 && Math.abs(this.#currentPxPerSecond) < 0.05) {
      this.#currentPxPerSecond = 0;
      this.#accumulator = 0;
      return 0;
    }

    this.#accumulator += this.#currentPxPerSecond * dt;
    const whole = Math.trunc(this.#accumulator);
    this.#accumulator -= whole;
    return whole;
  }
}

export const autoScrollStore = new AutoScrollStore();
