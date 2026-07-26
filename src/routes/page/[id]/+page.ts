import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAyahsForPage } from '$lib/api/db';

const MAX_PAGE = 604;

export const load: PageLoad = async ({ params }) => {
  const pageNumber = Number(params.id);
  if (!Number.isInteger(pageNumber) || pageNumber < 1 || pageNumber > MAX_PAGE) {
    error(404, 'Page not found');
  }

  try {
    const ayahs = await getAyahsForPage(pageNumber);
    return { pageNumber, ayahs, title: `Page ${pageNumber}` };
  } catch {
    error(404, `Page ${pageNumber} not found`);
  }
};
