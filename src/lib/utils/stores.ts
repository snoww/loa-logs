import { SearchFilter } from "$lib/types";
import { writable } from "svelte/store";

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