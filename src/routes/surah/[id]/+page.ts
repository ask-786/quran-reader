import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getSurah, getAyahsForSurah, getReadingPosition } from '$lib/api/db';
import { resumeTargetFor } from '$lib/utils/reading-scope';

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
    const [surah, ayahs, position] = await Promise.all([
      getSurah(surahId),
      getAyahsForSurah(surahId),
      getReadingPosition('surah', surahId),
    ]);

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
    };
  } catch {
    error(404, `Surah ${surahId} not found`);
  }
};
