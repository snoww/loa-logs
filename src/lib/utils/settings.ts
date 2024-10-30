import { classColors } from "$lib/constants/colors";
import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";
import { get, writable } from "svelte/store";
import { hideAll } from "tippy.js";
import { clickthroughStore } from "$lib/utils/stores";

export const defaultSettings = {
    general: {
        startLoaOnStart: false,
        lowPerformanceMode: false,
        showNames: true,
        showGearScore: false,
        hideNames: false,
        showEsther: true,
        hideLogo: false,
        showDate: true,
        showDifficulty: true,
        showGate: false,
        showDetails: false,
        showShields: false,
        showTanked: false,
        showBosses: false,
        splitLines: false,
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
        bossOnlyDamage: false,
        keepFavorites: true,
        hideMeterOnStart: false,
        hideLogsOnStart: false,
        constantLocalPlayerColor: false,
        bossOnlyDamageDefaultOn: true,
        startOnBoot: false,
        logsPerPage: 10
    },
    shortcuts: {
        hideMeter: {
            modifier: "Ctrl",
            key: "ArrowDown"
        },
        showLogs: {
            modifier: "Ctrl",
            key: "ArrowUp"
        },
        showLatestEncounter: {
            modifier: "Ctrl",
            key: "ArrowRight"
        },
        resetSession: {
            modifier: "",
            key: ""
        },
        pauseSession: {
            modifier: "",
            key: ""
        },
        manualSave: {
            modifier: "",
            key: ""
        },
        disableClickthrough: {
            modifier: "",
            key: ""
        }
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
        damage: false,
        dps: true,
        damagePercent: true,
        deathTime: false,
        critRate: true,
        critDmg: false,
        frontAtk: true,
        backAtk: true,
        counters: false,
        positionalDmgPercent: true,
        percentBuffBySup: false,
        percentIdentityBySup: false,
        percentBrand: false,
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
            percentBrand: false
        }
    },
    logs: {
        abbreviateHeader: false,
        splitPartyDamage: true,
        splitPartyBuffs: true,
        damage: true,
        dps: true,
        damagePercent: true,
        deathTime: true,
        critRate: true,
        critDmg: false,
        frontAtk: true,
        backAtk: true,
        counters: false,
        minEncounterDuration: 30,
        positionalDmgPercent: true,
        percentBuffBySup: false,
        percentIdentityBySup: false,
        percentBrand: false,
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
            percentBrand: false
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
                invoke("save_settings", { settings: value });
            }
            store.set(value);
        },
        update: store.update
    };
};

export const settings = settingsStore("settings", defaultSettings);
export const colors = settingsStore("classColors", classColors);
export const updateSettings = settingsStore("updateSettings", update);

export const miscSettings = settingsStore("miscSettings", {});

export async function registerShortcuts(shortcuts: any) {
    try {
        await unregisterAll();

        if (shortcuts.hideMeter.modifier && shortcuts.hideMeter.key) {
            await register(shortcuts.hideMeter.modifier + "+" + shortcuts.hideMeter.key, async () => {
                await invoke("toggle_meter_window");
                hideAll();
            });
        }
        if (shortcuts.showLogs.modifier && shortcuts.showLogs.key) {
            await register(shortcuts.showLogs.modifier + "+" + shortcuts.showLogs.key, async () => {
                await invoke("toggle_logs_window");
            });
        }
        if (shortcuts.showLatestEncounter.modifier && shortcuts.showLatestEncounter.key) {
            await register(
                shortcuts.showLatestEncounter.modifier + "+" + shortcuts.showLatestEncounter.key,
                async () => {
                    await invoke("open_most_recent_encounter");
                }
            );
        }
        if (shortcuts.resetSession.modifier && shortcuts.resetSession.key) {
            await register(shortcuts.resetSession.modifier + "+" + shortcuts.resetSession.key, async () => {
                await emit("reset-request");
            });
        }
        if (shortcuts.pauseSession.modifier && shortcuts.pauseSession.key) {
            await register(shortcuts.pauseSession.modifier + "+" + shortcuts.pauseSession.key, async () => {
                await emit("pause-request");
            });
        }
        if (shortcuts.manualSave.modifier && shortcuts.manualSave.key) {
            await register(shortcuts.manualSave.modifier + "+" + shortcuts.manualSave.key, async () => {
                await emit("save-request");
            });
        }

        if (shortcuts.disableClickthrough.modifier && shortcuts.disableClickthrough.key) {
            await register(
                shortcuts.disableClickthrough.modifier + "+" + shortcuts.disableClickthrough.key,
                async () => {
                    // if meter is clickthrough, disable it
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
            );
        }
    } catch (error) {
        await invoke("write_log", { message: "[live_meter::register_shortcuts]" + error });
    }
}

export const imagePath = settingsStore("imagePath", {});
export const skillIcon = settingsStore("skillIcon", {});
export const classIconCache = settingsStore("classIconCache", {});

export const keyboardKeys = [
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight"
];
