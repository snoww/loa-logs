import { encounterMap } from "$lib/constants/encounters";
import { SearchFilter } from "$lib/types";
import { readable, writable } from "svelte/store";

export const takingScreenshot = writable(false);

export const screenshotAlert = writable(false);
export const screenshotError = writable(false);

export const pageStore = writable(1);
export const searchStore = writable("");
export const backNavStore = writable(false);

export const ifaceChangedStore = writable(false);

export const searchFilter = writable(new SearchFilter());

export const selectedEncounters = writable(new Set<number>());

export const isWin11 = writable(false);

export const raidGates = readable(new Map<string, string>(), (set) => {
    const newMap = new Map<string, string>();

    Object.values(encounterMap).forEach((raid) => {
        Object.entries(raid).forEach(([gate, bosses]) => {
            bosses.forEach((boss) => {
                newMap.set(boss, gate);
            });
        });
    });

    set(newMap);

    return () => {};
});
