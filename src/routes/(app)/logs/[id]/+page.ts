import type { PageLoad } from "./$types";
import { loadEncounter } from "$lib/api";

export const prerender: boolean = false;

export const load: PageLoad = async ({ params }) => {
  const id = params.id;
  return await loadEncounter(id);
};
