import { classColors } from "$lib/constants/colors";
import { settings } from "$lib/stores.svelte";
import { clickthroughStore } from "$lib/utils/stores";
import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";
import { get, writable } from "svelte/store";
import { hideAll } from "tippy.js";

export const UWUOWO_URL = "https://uwuowo.mathi.moe";

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
    splitLines: true,
    underlineHovered: false,
    accentColor: "theme-pink",
    rawSocket: false,
    autoIface: true,
    ifDesc: "",
    ip: "",
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
    experimentalFeatures: false
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
    bossHp: true,
    bossHpBar: true,
    splitBossHpBar: false,
    abbreviateHeader: true,
    showTimeUntilKill: false,
    splitPartyBuffs: true,
    pinSelfParty: true,
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
    positionalDmgPercent: true,
    percentBuffBySup: false,
    percentIdentityBySup: false,
    percentBrand: false,
    percentHatBySup: false,
    rdpsSplitParty: true,
    rdpsDamageGiven: false,
    rdpsDamageReceived: false,
    rdpsContribution: false,
    rdpsSContribution: false,
    rdpsDContribution: false,
    rdpsSyn: true,
    rdpsSSyn: true,
    rdpsDSyn: true,
    ssyn: true,
    breakdown: {
      damage: true,
      dps: true,
      damagePercent: true,
      critRate: true,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: false,
      maxDamage: false,
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
    rdpsSplitParty: true,
    rdpsDamageGiven: true,
    rdpsDamageReceived: true,
    rdpsContribution: false,
    rdpsSContribution: true,
    rdpsDContribution: false,
    rdpsSyn: true,
    rdpssSyn: true,
    rdpsdSyn: true,
    ssyn: true,
    breakdown: {
      damage: true,
      dps: true,
      damagePercent: true,
      critRate: true,
      adjustedCritRate: false,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: false,
      maxDamage: false,
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
  buffs: {
    default: true
  },
  sync: {
    enabled: false,
    accessToken: "",
    validToken: false,
    auto: false,
    username: "",
    visibility: "0"
  }
};

export const defaultClassColors: Record<string, string> = {
  Local: "#FFC9ED",
  Berserker: "#ee2e48",
  Destroyer: "#7b9aa2",
  Gunlancer: "#e1907e",
  Paladin: "#ff9900",
  Slayer: "#db6a42",
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

export type Shortcut = {
  name: string;
  action: () => void | Promise<void>;
};
export const shortcuts: Record<string, Shortcut> = {
  hideMeter: {
    name: "Hide Meter",
    action: () => invoke("toggle_meter_window")
  },
  showLogs: {
    name: "Show Logs",
    action: () => invoke("toggle_logs_window")
  },
  showLatestEncounter: {
    name: "Show Latest Encounter",
    action: () => invoke("open_most_recent_encounter")
  },
  resetSession: {
    name: "Reset Session",
    action: () => emit("reset-request")
  },
  pauseSession: {
    name: "Pause Session",
    action: () => emit("pause-request")
  },
  manualSave: {
    name: "Manual Save",
    action: () => emit("save-request")
  },
  disableClickthrough: {
    name: "Disable Clickthrough",
    action: async () => {
      if (get(clickthroughStore)) {
        await invoke("set_clickthrough", { set: false });
        await invoke("write_log", { message: "disabling clickthrough" });
        clickthroughStore.update(() => false);
      } else {
        await invoke("set_clickthrough", { set: true });
        await invoke("write_log", { message: "enabling clickthrough" });
        clickthroughStore.update(() => true);
      }
    }
  }
};

export async function registerShortcuts() {
  try {
    for (const sc of Object.entries(shortcuts)) {
      const shortcut = settings.appSettings.shortcuts[sc[0] as keyof typeof settings.appSettings.shortcuts];
      if (shortcut) {
        await register(shortcut, () => {
          sc[1].action();
        });
      }
    }
  } catch (error) {
    await invoke("write_log", { message: "[live_meter::register_shortcuts] " + error });
  }
}

export const update = {
  available: false,
  manifest: undefined,
  dismissed: false,
  isNotice: false
};

const settingsStore = (key: string, defaultSettings: object) => {
  const storedSettings = localStorage.getItem(key);
  const value = storedSettings ? JSON.parse(storedSettings) : defaultSettings;
  const store = writable(value);
  if (typeof window !== "undefined") {
    window.addEventListener("storage", (event) => {
      if (event.key === key) {
        const newValue = JSON.parse(event.newValue || "");
        store.set(newValue);
      }
    });
  }
  return {
    subscribe: store.subscribe,
    set: (value: object) => {
      localStorage.setItem(key, JSON.stringify(value));
      if (key === "settings") {
        // invoke("save_settings", { settings: value });
      }
      store.set(value);
    },
    update: store.update
  };
};

export const updateSettings = settingsStore("updateSettings", update);

export const miscSettings = settingsStore("miscSettings", {});
