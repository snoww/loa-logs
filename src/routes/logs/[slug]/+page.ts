import { invoke } from '@tauri-apps/api/tauri';
import type { PageLoad } from './$types';
import type { Encounter } from '$lib/types';
 
export const load = (async ({ params }) => {
    const encounter: Encounter = await invoke("load_encounter", { id: params.slug });
    return {
        id: params.slug,
        encounter: encounter
    };
}) satisfies PageLoad;