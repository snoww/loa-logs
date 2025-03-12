import type { Encounter } from "$lib/types";
import { invoke } from "@tauri-apps/api/tauri";
import type { PageLoad } from "./$types";

export const prerender: boolean = false;

export const load: PageLoad = async ({ params }) => {
    // If there's no such id this return `Encounter` with all fields zeroed
    const encounter = (await invoke("load_encounter", { id: params.id })) as Encounter;
    return { id: params.id, encounter };
};
