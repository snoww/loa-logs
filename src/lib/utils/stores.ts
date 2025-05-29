import { encounterMap } from "$lib/constants/encounters";
import { SearchFilter, type SkillCastInfo } from "$lib/types";
import MarkdownIt from "markdown-it";
import { readable, writable, type Writable } from "svelte/store";

export const takingScreenshot = writable(false);

export const screenshotAlert = writable(false);
export const screenshotError = writable(false);

export const pageStore = writable(1);
export const searchStore = writable("");
export const backNavStore = writable(false);

export const ifaceChangedStore = writable(false);
export const uploadErrorStore = writable(false);
export const uploadErrorMessage = writable("");

export const searchFilter = writable(new SearchFilter());

export const selectedEncounters = writable(new Set<number>());

export const clickthroughStore = writable(false);
export const rdpsEventDetails = writable("");
export const localPlayer = writable("");
export const missingInfo = writable(false);
export const focusedSkillCast: Writable<SkillCastInfo> = writable({ skillId: 0, cast: 0 });

export const syncStore = writable({
  syncing: false,
  synced: 0,
  total: 0,
  message: "",
  stop: false
});
