import { loadEncounter } from "$lib/api";

import type { PageLoad } from "./$types";

export const prerender: boolean = false;

export const load: PageLoad = async ({ params }) => {
  const id = params.id;
  return await loadEncounter(id);
};
