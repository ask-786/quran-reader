/**
 * Turning a reading scope into something you can navigate to and read.
 *
 * A stored position is `(scope, scope_id, ayah_id)` — the range that was open
 * and where in it the reader was. Restoring one is therefore navigating to that
 * range's route and landing on that Ayah, which is what `scopeHref` builds.
 */

import { resolve } from '$app/paths';
import type { Ayah, AyahRef, ReadingPosition, ReadingScope, Surah } from '$lib/types/database';

/**
 * The query parameter that carries a resume target. A global Ayah id, unlike
 * the Surah route's `?ayah=`, which is an Ayah *number* within that Surah —
 * two different things, so they get two different names rather than one
 * parameter that means something different per route.
 */
export const RESUME_PARAM = 'resume';

/**
 * The route for a range, optionally deep-linked to a specific Ayah id.
 *
 * The page route takes no resume target: a Mushaf page *is* one screen, so
 * there is nowhere within it to resume to — opening the page is the whole job.
 */
export function scopeHref(scope: ReadingScope, scopeId: number, resumeAyahId?: number) {
  const id = String(scopeId);
  const at = scope === 'page' ? undefined : resumeAyahId;

  // Spelled out per route, with the query inside the template `resolve()` is
  // given: it types its result off a *literal* route pattern, so a path with a
  // separately appended query string is no longer a route `goto()` will take.
  switch (scope) {
    case 'surah':
      return at === undefined
        ? resolve('/surah/[id]', { id })
        : resolve(`/surah/[id]?${RESUME_PARAM}=${at}`, { id });
    case 'juz':
      return at === undefined
        ? resolve('/juz/[id]', { id })
        : resolve(`/juz/[id]?${RESUME_PARAM}=${at}`, { id });
    case 'hizb':
      return at === undefined
        ? resolve('/hizb/[id]', { id })
        : resolve(`/hizb/[id]?${RESUME_PARAM}=${at}`, { id });
    case 'page':
      return resolve('/page/[id]', { id });
  }
}

/**
 * Which Ayah a freshly loaded range should open on, given its URL and its
 * stored position. An explicit `?resume=` (a history entry being reopened)
 * outranks the stored position, and anything outside the loaded range is
 * discarded rather than handed to the reader as an unreachable target.
 *
 * Returns `undefined` for the first Ayah of the range: the views centre their
 * target, so asking for the opening Ayah would push the range's own beginning
 * into the top half of the viewport — which is not where "start here" looks.
 */
export function resumeTargetFor(
  url: URL,
  ayahs: Ayah[],
  position: ReadingPosition | null,
): number | undefined {
  const requested = Number(url.searchParams.get(RESUME_PARAM));
  const candidate = Number.isInteger(requested) && requested > 0 ? requested : position?.ayah.id;

  if (candidate === undefined || ayahs.length === 0) return undefined;
  if (candidate <= ayahs[0].id || candidate > ayahs[ayahs.length - 1].id) return undefined;
  return candidate;
}

/**
 * What to call a range in a list. Surahs go by name — "Al-Baqarah" is how a
 * reader thinks of one, where "Surah 2" is how the database does — and fall
 * back to the number while the Surah list is still loading.
 */
export function scopeLabel(
  scope: ReadingScope,
  scopeId: number,
  surahs: { get(id: number): Surah | undefined },
): string {
  switch (scope) {
    case 'surah':
      return surahs.get(scopeId)?.transliteration ?? `Surah ${scopeId}`;
    case 'juz':
      return `Juz ${scopeId}`;
    case 'hizb':
      return `Hizb ${scopeId}`;
    case 'page':
      return `Page ${scopeId}`;
  }
}

/** The conventional Surah:Ayah citation, e.g. `2:255`. */
export function ayahRefLabel(ref: AyahRef): string {
  return `${ref.surah_id}:${ref.ayah_number}`;
}

/**
 * How far a sitting got, as one line. A sitting that never moved off its
 * opening Ayah reads as a single reference rather than "2:255 → 2:255", and one
 * that stayed inside a single Surah drops the repeated Surah number from the
 * far end — "2:142 → 176" is how the range would be written out loud.
 */
export function rangeLabel(start: AyahRef, end: AyahRef): string {
  if (start.id === end.id) return ayahRefLabel(start);
  if (start.surah_id === end.surah_id) return `${ayahRefLabel(start)} → ${end.ayah_number}`;
  return `${ayahRefLabel(start)} → ${ayahRefLabel(end)}`;
}

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * A coarse "when" for a history row. Coarse on purpose: the useful question is
 * whether this was the reading you just put down or one from last week, and an
 * exact timestamp on every row makes the list harder to skim, not easier.
 */
export function relativeTime(iso: string, now = Date.now()): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return '';

  // Clock changes and a coarse `strftime('…','now')` can both land a stamp a
  // moment in the future; that is "just now", not a negative age.
  const elapsed = Math.max(0, now - then);
  if (elapsed < MINUTE) return 'just now';
  if (elapsed < HOUR) {
    const mins = Math.floor(elapsed / MINUTE);
    return `${mins} min ago`;
  }
  if (elapsed < DAY) {
    const hours = Math.floor(elapsed / HOUR);
    return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
  }
  if (elapsed < 7 * DAY) {
    const days = Math.floor(elapsed / DAY);
    return days === 1 ? 'yesterday' : `${days} days ago`;
  }
  return new Date(then).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });
}
