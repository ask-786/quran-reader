/**
 * Reports which Ayah sits under the vertical middle of a scroll container.
 *
 * The reader restores a position with `scrollIntoView({ block: 'center' })`, so
 * reading the position back off the same centre line is what makes a
 * record-then-restore round trip land where it started.
 *
 * `rootMargin` collapses the root box down to a thin band across that centre
 * line; only elements crossing it are ever candidates. Ratio thresholds can't
 * do this job — an element taller than the viewport never reaches one.
 */
export function observeCenteredAyah(
  root: HTMLElement,
  targets: Iterable<Element>,
  attribute: string,
  onCenter: (ayahId: number) => void,
): IntersectionObserver {
  // IntersectionObserver only reports elements whose state *changed*, so track
  // the currently-crossing set to pick the closest one to the exact centre.
  const crossing = new Set<Element>();

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) crossing.add(entry.target);
        else crossing.delete(entry.target);
      }
      if (crossing.size === 0) return;

      const rootRect = root.getBoundingClientRect();
      const centerY = rootRect.top + rootRect.height / 2;
      let best: Element | null = null;
      let bestDistance = Infinity;
      for (const el of crossing) {
        const rect = el.getBoundingClientRect();
        const distance = Math.abs(rect.top + rect.height / 2 - centerY);
        if (distance < bestDistance) {
          bestDistance = distance;
          best = el;
        }
      }

      const raw = best?.getAttribute(attribute);
      const id = raw === null || raw === undefined ? NaN : Number(raw);
      if (Number.isFinite(id)) onCenter(id);
    },
    { root, rootMargin: '-49% 0px -49% 0px', threshold: 0 },
  );

  for (const el of targets) observer.observe(el);
  return observer;
}
