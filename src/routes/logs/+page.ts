import { classNameToClassId } from "$lib/constants/classes";
import { encounterMap } from "$lib/constants/encounters";
import type { EncountersOverview, SearchFilter } from "$lib/types";
import { settings as settingsStore } from "$lib/utils/settings";
import { pageStore, searchFilter as searchFilterStore, searchStore } from "$lib/utils/stores";
import { invoke } from "@tauri-apps/api/tauri";
import { get } from "svelte/store";

export async function load() {
    return await _loadEncountersOverview(get(searchFilterStore), get(searchStore), get(pageStore));
}

export async function _loadEncountersOverview(searchFilter: SearchFilter, search: string, page: number) {
    const bosses = Array.from(searchFilter.bosses);
    if (searchFilter.encounters.size > 0) {
        for (const encounter of searchFilter.encounters) {
            const raid = encounter.substring(0, encounter.indexOf(" "));
            bosses.push(...encounterMap[raid][encounter]);
        }
    }
    // start or space (^|\s) + word (\w+) + colon or space or end (:|\s|$)
    // using lookbehind (?<=) and lookahead (?=) https://regex101.com/r/1cMFH8/4
    // if word is a valid className, replace it with the classId
    // example: "bard:Anyduck shadowhunter" -> "204:Anyduck 403"
    const searchQuery = search.replace(/(?<=^|\s)\w+(?=:|\s|$)/g, (word: string) => {
        const className = word[0].toUpperCase() + word.substring(1).toLowerCase();
        return String(classNameToClassId[className] || word);
    });

    return await invoke<EncountersOverview>("load_encounters_preview", {
        page: page,
        pageSize: get(settingsStore).general.logsPerPage,
        search: searchQuery,
        filter: {
            minDuration: searchFilter.minDuration,
            bosses: bosses,
            cleared: searchFilter.cleared,
            favorite: searchFilter.favorite,
            difficulty: searchFilter.difficulty,
            bossOnlyDamage: searchFilter.bossOnlyDamage,
            sort: searchFilter.sort,
            order: searchFilter.order
        }
    });
}
