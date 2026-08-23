import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAyahsForJuz, getReadingPosition } from '$lib/api/db';
import { resumeTargetFor } from '$lib/utils/reading-scope';

export const load: PageLoad = async ({ params, url }) => {
  const juz = Number(params.id);
  if (!Number.isInteger(juz) || juz < 1 || juz > 30) {
    error(404, 'Juz not found');
  }

  try {
    const [ayahs, position] = await Promise.all([
      getAyahsForJuz(juz),
      getReadingPosition('juz', juz),
    ]);
    return {
      juz,
      ayahs,
      title: `Juz ${juz}`,
      resumeAyahId: resumeTargetFor(url, ayahs, position),
    };
  } catch {
    error(404, `Juz ${juz} not found`);
  }
};
