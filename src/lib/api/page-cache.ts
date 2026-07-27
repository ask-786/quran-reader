/**
 * Process-wide cache of Mushaf page layout data.
 *
 * Both reading views are built from the same `page_line` rows, and both are
 * destroyed and rebuilt every time the Mushaf/list toggle flips — so without a
 * cache above the component tree, every toggle refetched the entire open
 * Surah's page range (48 pages for Al-Baqara). The data is small and immutable
 * (it's the printed layout of the Mushaf, not user state), so it is held for
 * the app's lifetime up to an LRU cap.
 */

import { getPages } from './db';
import type { MushafPage } from '$lib/types/database';

/**
 * Roughly two Al-Baqara-sized ranges. Page data averages ~1.5KB serialized, so
 * the whole cache at cap is a couple of MB — trivial next to the page fonts,
 * which are what actually needed bounding (see mushaf-fonts.ts).
 */
const MAX_PAGES = 150;

/** Insertion order is the LRU order; a hit re-inserts to move it to the end. */
const cache = new Map<number, MushafPage>();
/** Ranges currently being fetched, so overlapping callers share one IPC call. */
const inFlight = new Map<string, Promise<void>>();

function touch(page: number): MushafPage | undefined {
  const hit = cache.get(page);
  if (hit) {
    cache.delete(page);
    cache.set(page, hit);
  }
  return hit;
}

function evict() {
  while (cache.size > MAX_PAGES) {
    const oldest = cache.keys().next();
    if (oldest.done) break;
    cache.delete(oldest.value);
  }
}

/**
 * Resolve an inclusive page range, fetching only the pages not already cached.
 * Returned pages are in ascending page order; pages the Mushaf doesn't have
 * are simply absent.
 */
export async function loadPages(start: number, end: number): Promise<MushafPage[]> {
  const missing: number[] = [];
  for (let p = start; p <= end; p++) if (!cache.has(p)) missing.push(p);

  if (missing.length > 0) {
    // One request per contiguous run of missing pages. In the common cases
    // (nothing cached, or a window extending past what is) that's a single
    // call; a fragmented range costs a few, still far short of one per page.
    const runs: Array<[number, number]> = [];
    for (const p of missing) {
      const last = runs[runs.length - 1];
      if (last && p === last[1] + 1) last[1] = p;
      else runs.push([p, p]);
    }

    await Promise.all(
      runs.map(([a, b]) => {
        const key = `${a}-${b}`;
        let pending = inFlight.get(key);
        if (!pending) {
          pending = getPages(a, b)
            .then((fetched) => {
              for (const page of fetched) cache.set(page.page, page);
              evict();
            })
            .finally(() => inFlight.delete(key));
          inFlight.set(key, pending);
        }
        return pending;
      }),
    );
  }

  const out: MushafPage[] = [];
  for (let p = start; p <= end; p++) {
    const hit = touch(p);
    if (hit) out.push(hit);
  }
  return out;
}

/** Cached pages only — no fetch. Used to render synchronously on a warm open. */
export function peekPages(start: number, end: number): MushafPage[] {
  const out: MushafPage[] = [];
  for (let p = start; p <= end; p++) {
    const hit = cache.get(p);
    if (hit) out.push(hit);
  }
  return out;
}

/** Whether every page in the range is already cached. */
export function hasPages(start: number, end: number): boolean {
  for (let p = start; p <= end; p++) if (!cache.has(p)) return false;
  return true;
}
