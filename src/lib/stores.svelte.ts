import { browser } from "$app/environment";
import { invoke } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { time } from "echarts/core";
import MarkdownIt from "markdown-it";
import { SvelteSet } from "svelte/reactivity";
import { readable } from "svelte/store";

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
  app = $state(defaultSettings);
  sync = $state(syncSettings);
  classColors = $state<Record<string, string>>(defaultClassColors);
  version = $state("");
  lockUpdate = false;

  constructor() {
    if (!browser) return;

    if (localStorage) {
      const updateSettings = (settings: string | null, init = false) => {
        this.lockUpdate = true;
        if (settings) {
          try {
            const settingsFromStorage = JSON.parse(settings) as LogSettings;
            mergeSettings(this.app, settingsFromStorage);
            if (!init) {
              invoke("save_settings", { settings: this.app });
            }
          } catch (e) {
            console.error(e);
          }
        }
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

      updateSettings(localStorage.getItem("appSettings"), true);
      updateClassColors(localStorage.getItem("classColors"));
      updateSyncSettings(localStorage.getItem("syncSettings"));
      updateVersion(localStorage.getItem("version"));

      $effect.root(() => {
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("appSettings", JSON.stringify(this.app));
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

      window.addEventListener("storage", (e) => {
        if (this.lockUpdate) return;
        const { key, newValue, storageArea } = e;
        if (storageArea !== localStorage) return;
        if (key === "appSettings") updateSettings(newValue);
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
  minDuration = $derived(settings.app.logs.minEncounterDuration);

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

export type LogSettings = typeof defaultSettings;
export const defaultSettings = {
  general: {
    startLoaOnStart: false,
    lowPerformanceMode: false,
    showNames: true,
    showGearScore: true,
    hideNames: false,
    showEsther: true,
    hideLogo: false,
    showDate: true,
    showDifficulty: true,
    showGate: false,
    showDetails: false,
    showShields: true,
    showTanked: false,
    showBosses: false,
    showRaidsOnly: true,
    splitLines: true,
    underlineHovered: false,
    accentColor: "theme-violet",
    autoIface: true,
    port: 6040,
    blur: true,
    blurWin11: false,
    isWin11: false,
    transparent: true,
    scale: "1",
    logScale: "1",
    alwaysOnTop: true,
    bossOnlyDamage: true,
    keepFavorites: true,
    hideMeterOnStart: false,
    hideLogsOnStart: false,
    constantLocalPlayerColor: false,
    bossOnlyDamageDefaultOn: true,
    startOnBoot: false,
    logsPerPage: 10,
    experimentalFeatures: false,
    mini: false,
    miniEdit: true,
    autoShow: false,
    autoHideDelay: 5
  },
  shortcuts: {
    hideMeter: "Control+ArrowDown",
    showLogs: "Control+ArrowUp",
    showLatestEncounter: "",
    resetSession: "",
    pauseSession: "",
    manualSave: "",
    disableClickthrough: ""
  },
  meter: {
    bossInfo: true,
    bossHpBar: true,
    splitBossHpBar: false,
    showTimeUntilKill: false,
    splitPartyBuffs: true,
    showClassColors: true,
    profileShortcut: false,
    damage: false,
    dps: true,
    damagePercent: true,
    deathTime: false,
    incapacitatedTime: false,
    critRate: true,
    critDmg: false,
    frontAtk: true,
    backAtk: true,
    counters: false,
    pinSelfParty: false,
    positionalDmgPercent: true,
    percentBuffBySup: true,
    percentIdentityBySup: true,
    percentBrand: true,
    percentHatBySup: true,
    breakdown: {
      damage: true,
      dps: true,
      damagePercent: true,
      critRate: true,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: false,
      maxDamage: true,
      casts: true,
      cpm: true,
      hits: false,
      hpm: false,
      percentBuffBySup: false,
      percentIdentityBySup: false,
      percentBrand: false,
      percentHatBySup: false
    }
  },
  mini: {
    info: "damage",
    bossHpBar: false
  },
  logs: {
    abbreviateHeader: false,
    splitPartyDamage: true,
    splitPartyBuffs: true,
    profileShortcut: true,
    damage: true,
    dps: true,
    damagePercent: true,
    deathTime: true,
    incapacitatedTime: true,
    critRate: true,
    critDmg: false,
    frontAtk: true,
    backAtk: true,
    counters: true,
    minEncounterDuration: 30,
    positionalDmgPercent: true,
    percentBuffBySup: true,
    percentIdentityBySup: true,
    percentHatBySup: true,
    percentBrand: true,
    breakdown: {
      damage: true,
      dps: true,
      damagePercent: true,
      critRate: true,
      adjustedCritRate: true,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: true,
      maxDamage: true,
      casts: true,
      cpm: true,
      hits: true,
      hpm: true,
      percentBuffBySup: false,
      percentIdentityBySup: false,
      percentBrand: false,
      percentHatBySup: false
    }
  },
  buffs: {
    default: true
  }
};

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
