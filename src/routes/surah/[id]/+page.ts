import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getSurah, getAyahsForSurah } from '$lib/api/db';

export const load: PageLoad = async ({ params }) => {
  const surahId = Number(params.id);
  if (!Number.isInteger(surahId) || surahId < 1 || surahId > 114) {
    error(404, 'Surah not found');
  }

  try {
    const [surah, ayahs] = await Promise.all([getSurah(surahId), getAyahsForSurah(surahId)]);
    return { surah, ayahs };
  } catch {
    error(404, `Surah ${surahId} not found`);
  }
};
