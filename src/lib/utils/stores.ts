import { encounterMap } from "$lib/constants/encounters";
import { SearchFilter } from "$lib/types";
import { readable, writable } from "svelte/store";
import MarkdownIt from "markdown-it";

export const takingScreenshot = writable(false);

export const screenshotAlert = writable(false);
export const screenshotError = writable(false);

export const pageStore = writable(1);
export const searchStore = writable("");
export const backNavStore = writable(false);

export const ifaceChangedStore = writable(false);

export const searchFilter = writable(new SearchFilter());

export const selectedEncounters = writable(new Set<number>());

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

export const clickthroughStore = writable(false);
export const rdpsEventDetails = writable("");
export const localPlayer = writable("");

const md = new MarkdownIt({
    html: true
});

// Remember the old renderer if overridden, or proxy to the default renderer.
const defaultRender = md.renderer.rules.link_open || function (tokens, idx, options, env, self) {
    return self.renderToken(tokens, idx, options);
};

md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
    // Add a new `target` attribute, or replace the value of the existing one.
    tokens[idx].attrSet('target', '_blank');

    // Pass the token to the default renderer.
    return defaultRender(tokens, idx, options, env, self);
};

export const markdownIt = readable(md);
