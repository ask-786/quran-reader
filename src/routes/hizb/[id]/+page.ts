import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAyahsForHizb, getReadingPosition } from '$lib/api/db';
import { resumeTargetFor } from '$lib/utils/reading-scope';

export const load: PageLoad = async ({ params, url }) => {
  const hizb = Number(params.id);
  if (!Number.isInteger(hizb) || hizb < 1 || hizb > 60) {
    error(404, 'Hizb not found');
  }

  try {
    const [ayahs, position] = await Promise.all([
      getAyahsForHizb(hizb),
      getReadingPosition('hizb', hizb),
    ]);
    return {
      hizb,
      ayahs,
      title: `Hizb ${hizb}`,
      resumeAyahId: resumeTargetFor(url, ayahs, position),
    };
  } catch {
    error(404, `Hizb ${hizb} not found`);
  }
};
