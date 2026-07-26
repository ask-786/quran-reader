import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAyahsForJuz } from '$lib/api/db';

export const load: PageLoad = async ({ params }) => {
  const juz = Number(params.id);
  if (!Number.isInteger(juz) || juz < 1 || juz > 30) {
    error(404, 'Juz not found');
  }

  try {
    const ayahs = await getAyahsForJuz(juz);
    return { juz, ayahs, title: `Juz ${juz}` };
  } catch {
    error(404, `Juz ${juz} not found`);
  }
};
