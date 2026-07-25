export const AUTO_SCROLL_SPEED_MIN = 1;
export const AUTO_SCROLL_SPEED_MAX = 10;

const DEFAULT_SPEED = 4;
const PX_PER_SECOND_PER_STEP = 15;

class AutoScrollStore {
  active = $state(false);
  speed = $state(DEFAULT_SPEED);

  get pxPerSecond() {
    return this.speed * PX_PER_SECOND_PER_STEP;
  }

  toggle() {
    this.active = !this.active;
  }

  stop() {
    this.active = false;
  }

  setSpeed(value: number) {
    this.speed = Math.min(
      AUTO_SCROLL_SPEED_MAX,
      Math.max(AUTO_SCROLL_SPEED_MIN, Math.round(value)),
    );
  }
}

export const autoScrollStore = new AutoScrollStore();
