import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAyahsForHizb } from '$lib/api/db';

export const load: PageLoad = async ({ params }) => {
  const hizb = Number(params.id);
  if (!Number.isInteger(hizb) || hizb < 1 || hizb > 60) {
    error(404, 'Hizb not found');
  }

  try {
    const ayahs = await getAyahsForHizb(hizb);
    return { hizb, ayahs, title: `Hizb ${hizb}` };
  } catch {
    error(404, `Hizb ${hizb} not found`);
  }
};
