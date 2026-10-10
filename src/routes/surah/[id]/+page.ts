import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getSurah, getAyahsForSurah, getReadingPosition } from '$lib/api/db';
import { resumeTargetFor } from '$lib/utils/reading-scope';
import { RESTART_PARAM, ROUTINE_PARAM, routinesStore } from '$lib/stores/routines.svelte';

export const load: PageLoad = async ({ params, url }) => {
  const surahId = Number(params.id);
  if (!Number.isInteger(surahId) || surahId < 1 || surahId > 114) {
    error(404, 'Surah not found');
  }

  try {
    // The stored position is fetched here rather than read out of a store in
    // the component, so it is settled before the reader mounts. The old
    // settings-based resume raced its own store's async load and lost on a
    // cold start — the reader had already positioned itself by the time the
    // position arrived.
    const routineId = Number(url.searchParams.get(ROUTINE_PARAM));
    const restart = url.searchParams.has(RESTART_PARAM);
    const [surah, ayahs, stored, routine] = await Promise.all([
      getSurah(surahId),
      getAyahsForSurah(surahId),
      getReadingPosition('surah', surahId),
      Number.isInteger(routineId) && routineId > 0
        ? routinesStore.plan(routineId, surahId, restart)
        : null,
    ]);

    // A routine is read from the top of each Surah every day, so the stored
    // position only counts if it was written during this run — which is what
    // brings a reader who closed the app mid-Surah back to the same Ayah, and
    // only them. A run about to start has nothing written in it yet. Both
    // stamps are the same UTC ISO shape, so they compare as strings.
    const position =
      routine &&
      stored &&
      (routine.runStartedAt === null || stored.updated_at < routine.runStartedAt)
        ? null
        : stored;

    // `?ayah=` is an Ayah *number* within this Surah — the shareable form of a
    // deep link — and outranks both `?resume=` and the stored position.
    const ayahParam = url.searchParams.get('ayah');
    const linked = ayahParam
      ? ayahs.find((a) => a.ayah_number === Number(ayahParam))?.id
      : undefined;

    return {
      surah,
      ayahs,
      title: surah.transliteration,
      resumeAyahId: linked ?? resumeTargetFor(url, ayahs, position),
      // Entered by the page once the navigation lands, not here — see `plan`.
      routine: routine ? { id: routineId, restart } : null,
    };
  } catch {
    error(404, `Surah ${surahId} not found`);
  }
};
