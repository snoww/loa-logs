import type { Encounter } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import type { PageLoad } from "./$types";

export const prerender: boolean = false;

export const load: PageLoad = async ({ params }) => {
  const encounter = (await invoke("load_encounter", { id: params.id })) as Encounter;
  return encounter;
};
