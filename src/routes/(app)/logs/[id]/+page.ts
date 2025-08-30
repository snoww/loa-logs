import { loadEncounter } from "$lib/api";
import type { PageLoad } from "./$types";

export const prerender: boolean = false;

export const load: PageLoad = async ({ params }) => {
  const encounter = loadEncounter(params.id);
  return encounter;
};
