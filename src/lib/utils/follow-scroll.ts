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
  const direct = document.querySelector<HTMLElement>(`[data-ayah-id="${ayahId}"]`);
  if (direct) {
    direct.scrollIntoView({ block: 'center', behavior: 'smooth' });
    return;
  }

  const lines = document.querySelectorAll<HTMLElement>('[data-line-ayah-id]');
  let best: HTMLElement | null = null;
  for (const line of lines) {
    const id = Number(line.dataset.lineAyahId);
    if (!Number.isFinite(id) || id > ayahId) continue;
    // Lines are in document order, so the last one at or before the verse is
    // the closest one before it.
    best = line;
  }
  best?.scrollIntoView({ block: 'center', behavior: 'smooth' });
}
