/**
 * The Ayah currently under the middle of the reader viewport, kept up to date
 * by whichever view is mounted (Mushaf page view or the Ayah list).
 *
 * Toggling reading mode destroys one view and mounts the other, so there is no
 * component state to carry the position across — this store is what survives
 * the swap and lets the incoming view resume on the same Ayah.
 */
class ReaderPositionStore {
  ayahId = $state<number | null>(null);

  reset() {
    this.ayahId = null;
  }
}

export const readerPosition = new ReaderPositionStore();
