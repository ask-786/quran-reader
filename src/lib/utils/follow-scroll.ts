/**
 * Bring the verse being recited into view, in whichever reading view is up.
 *
 * One function for both, because the two views anchor Ayahs differently and the
 * player should not have to know which is mounted:
 *
 * - The scrolling reader gives every row a `data-ayah-id`, rendered or
 *   windowed, so the first lookup always finds it there.
 * - The Mushaf page view puts `data-ayah-id` on word spans, which exist only
 *   for pages currently rendered, and tags every line with the Ayah it opens on
 *   (`data-line-ayah-id`) whether or not its glyphs are loaded. So the fallback
 *   is the last line that opens at or before this verse — the line the verse is
 *   on, or the one it started on when it spans several.
 */
export function scrollToRecitedAyah(ayahId: number) {
  findAyahElement(ayahId)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
}

/**
 * The same, but only when the verse is outside the reader's visible area — for
 * stepping the tafsir card from verse to verse, where a reader who can already
 * see the verse should not have the page move under them on every press.
 */
export function revealAyah(ayahId: number) {
  const el = findAyahElement(ayahId);
  if (!el) return;
  const box = el.getBoundingClientRect();
  const view = scrollParent(el)?.getBoundingClientRect() ?? {
    top: 0,
    bottom: window.innerHeight,
  };
  if (box.top >= view.top && box.bottom <= view.bottom) return;
  el.scrollIntoView({ block: 'center', behavior: 'smooth' });
}

function scrollParent(el: HTMLElement): HTMLElement | null {
  for (let node = el.parentElement; node; node = node.parentElement) {
    const { overflowY } = getComputedStyle(node);
    if (overflowY === 'auto' || overflowY === 'scroll') return node;
  }
  return null;
}

/** See the note on `scrollToRecitedAyah` for why there are two lookups. */
function findAyahElement(ayahId: number): HTMLElement | null {
  const direct = document.querySelector<HTMLElement>(`[data-ayah-id="${ayahId}"]`);
  if (direct) return direct;

  const lines = document.querySelectorAll<HTMLElement>('[data-line-ayah-id]');
  let best: HTMLElement | null = null;
  for (const line of lines) {
    const id = Number(line.dataset.lineAyahId);
    if (!Number.isFinite(id) || id > ayahId) continue;
    // Lines are in document order, so the last one at or before the verse is
    // the closest one before it.
    best = line;
  }
  return best;
}
