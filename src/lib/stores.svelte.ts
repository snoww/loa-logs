import { browser } from "$app/environment";
import { invoke } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { time } from "echarts/core";
import MarkdownIt from "markdown-it";
import { SvelteSet } from "svelte/reactivity";
import { readable } from "svelte/store";
import type { AppSettings } from "./settings";

/**
 * Merge settings from local storage into default settings.
 */
export const mergeSettings = (defaultSettings: any, storageSettings: any) => {
  for (const key of Object.keys(storageSettings)) {
    if (key in defaultSettings) {
      if (typeof storageSettings[key] === "object" && storageSettings[key] !== null) {
        mergeSettings(defaultSettings[key], storageSettings[key]);
      } else {
        defaultSettings[key] = storageSettings[key];
      }
    }
  }
};

class Settings {
  app: AppSettings = $state({} as any)!;
  sync = $state(syncSettings);
  classColors = $state<Record<string, string>>(defaultClassColors);
  version = $state("");
  lockUpdate = false;

  set(settings: AppSettings) {
    Object.assign(this.app, settings);
  }

  constructor() {
    if (!browser) return;

    if (localStorage) {
      const updateSettings = (settings: AppSettings) => {
        this.lockUpdate = true;
        invoke("save_settings", { settings });
        this.lockUpdate = false;
      };

      const updateClassColors = (classColors: string | null) => {
        this.lockUpdate = true;
        if (classColors) {
          try {
            const colors = JSON.parse(classColors);
            for (const [key, value] of Object.entries(colors)) {
              if (typeof value === "string" && Object.hasOwn(this.classColors, key)) {
                this.classColors[key] = value;
              }
            }
          } catch (e) {}
        }
        this.lockUpdate = false;
      };

      const updateSyncSettings = (settings: string | null) => {
        this.lockUpdate = true;
        if (settings) {
          try {
            const syncSettings = JSON.parse(settings) as SyncSettings;
            mergeSettings(this.sync, syncSettings);
          } catch (e) {}
        }
        this.lockUpdate = false;
      };

      const updateVersion = async (newVersion: string | null) => {
        this.lockUpdate = true;
        if (newVersion) {
          this.version = newVersion;
        }
        this.lockUpdate = false;
      };

      updateClassColors(localStorage.getItem("classColors"));
      updateSyncSettings(localStorage.getItem("syncSettings"));
      updateVersion(localStorage.getItem("version"));

      let hasLoaded = $derived(Object.keys(this.app).length > 0);

      $effect.root(() => {
        $effect(() => {
          if(!hasLoaded) {
            return;
          }
          updateSettings(this.app);
        });
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("classColors", JSON.stringify(this.classColors));
        });
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("syncSettings", JSON.stringify(this.sync));
        });
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("version", this.version);
        });
      });

      window.addEventListener("storage", (event: StorageEvent) => {
        if (this.lockUpdate) return;
        const { key, newValue, storageArea } = event;
        if (storageArea !== localStorage) return;

        else if (key === "classColors") updateClassColors(newValue);
        else if (key === "syncSettings") updateSyncSettings(newValue);
        else if (key === "version") updateVersion(newValue);
        else return;
      });
    } else {
      console.warn("localStorage not available?");
    }
  }
}

export type sortColumns = "id" | "my_dps" | "duration";
export type sortOrder = "asc" | "desc";

export class EncounterFilter {
  search = $state("");
  page = $state(1);
  bosses = $state(new SvelteSet<string>());
  encounters = $state(new SvelteSet<string>());
  favorite = $state(false);
  cleared = $state(false);
  difficulty = $state("");
  sort: sortColumns = $state("id");
  order: sortOrder = $state("desc");
  minDuration = $derived(30);

  setMinDuration(minDuration: number) {
    this.minDuration = minDuration;
  }

  reset() {
    this.search = "";
    this.page = 1;
    this.bosses = new SvelteSet();
    this.encounters = new SvelteSet();
    this.favorite = false;
    this.cleared = false;
    this.difficulty = "";
    this.sort = "id";
    this.order = "desc";
  }
}

export type SyncSettings = typeof syncSettings;
export const syncSettings = {
  accessToken: "",
  validToken: false,
  auto: false,
  visibility: "0"
};

export const defaultClassColors: Record<string, string> = {
  Local: "#FFC9ED",
  Berserker: "#ee2e48",
  Destroyer: "#7b9aa2",
  Gunlancer: "#e1907e",
  Paladin: "#ff9900",
  Slayer: "#db6a42",
  Valkyrie: "#ffbf00",
  Arcanist: "#b38915",
  Summoner: "#22aa99",
  Bard: "#674598",
  Sorceress: "#66aa00",
  Wardancer: "#aaaa11",
  Scrapper: "#990099",
  Soulfist: "#316395",
  Glaivier: "#f6da6a",
  Striker: "#994499",
  Breaker: "#4de3d1",
  Deathblade: "#a91a16",
  Shadowhunter: "#0099c6",
  Reaper: "#109618",
  Souleater: "#c16ed0",
  Sharpshooter: "#dd4477",
  Deadeye: "#4442a8",
  Artillerist: "#33670b",
  Machinist: "#3b4292",
  Gunslinger: "#6bcec2",
  Artist: "#a34af0",
  Aeromancer: "#084ba3",
  Wildsoul: "#3a945e"
};

export class Misc {
  liveConnectionListening = $state(false);
  raidInProgress = $state(false);
  reset = $state(false);
  paused = $state(false);
  missingInfo = $state(false);
  clickthrough = $state(false);
  modifyingShortcuts = $state(false);
}

export class SyncProgress {
  syncing = $state(false);
  uploaded = $state(0);
  total = $state(0);
  message = $state("");
  stop = $state(false);
}

export class SkillCastInfo {
  skillId = $state(0);
  cast = $state(0);
}

export class UpdateInfo {
  available = $state(false);
  manifest: Update | undefined = $state(undefined);
}

export const settings = new Settings();
export const encounterFilter = new EncounterFilter();
export const misc = new Misc();
export const syncProgress = new SyncProgress();
export const focusedCast = new SkillCastInfo();
export const screenshot = (() => {
  let state = $state(false);
  return {
    get state() {
      return state;
    },
    take() {
      state = true;
    },
    done() {
      state = false;
    }
  };
})();
export const updateInfo = new UpdateInfo();

const md = new MarkdownIt({
  html: true
});

// Remember the old renderer if overridden, or proxy to the default renderer.
const defaultRender =
  md.renderer.rules.link_open ||
  function (tokens, idx, options, env, self) {
    return self.renderToken(tokens, idx, options);
  };

md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  // Add a new `target` attribute, or replace the value of the existing one.
  tokens[idx].attrSet("target", "_blank");

  // Pass the token to the default renderer.
  return defaultRender(tokens, idx, options, env, self);
};

export const markdownIt = md;
