import { invoke } from '@tauri-apps/api';
import { register, unregisterAll } from '@tauri-apps/api/globalShortcut';
import { writable } from 'svelte/store';

export const defaultSettings = {
    "general": {
        "showNames": true,
        "showEsther": true,
        "accentColor": "theme-pink",
        "rawSocket": false,
        "port": 6040,
        "blur": true,
    },
    "shortcuts": {
        "hideMeter": {
            "modifier": "Ctrl",
            "key": "ArrowDown",
        },
        "showLogs": {
            "modifier": "Ctrl",
            "key": "ArrowUp",
        },
        "showLatestEncounter": {
            "modifier": "Ctrl",
            "key": "ArrowRight",
        }
    },
    "meter": {
        "bossHp": true,
        "damage": false,
        "dps": true,
        "damagePercent": true,
        "deathTime": false,
        "critRate": true,
        "frontAtk": true,
        "backAtk": true,
        "counters": false,
        "breakdown": {
            "damage": true,
            "dps": true,
            "damagePercent": true,
            "critRate": true,
            "frontAtk": true,
            "backAtk": true,
            "avgDamage": false,
            "maxDamage": false,
            "casts": true,
            "hits": false,
        }
    },
    "logs": {
        "damage": true,
        "dps": true,
        "damagePercent": true,
        "deathTime": true,
        "critRate": true,
        "frontAtk": true,
        "backAtk": true,
        "counters": false,
        "minEncounterDuration": 30,
        "breakdown": {
            "damage": true,
            "dps": true,
            "damagePercent": true,
            "critRate": true,
            "frontAtk": true,
            "backAtk": true,
            "avgDamage": false,
            "maxDamage": false,
            "casts": true,
            "hits": false,
        }
    }
};

const settingsStore = (key: string, defaultSettings: object) => {
    const storedSettings = localStorage.getItem(key);
    const value = storedSettings ? JSON.parse(storedSettings) : defaultSettings;
    const store = writable(value);
    if (typeof window !== 'undefined') {
        window.addEventListener('storage', (event) => {
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

export async function registerShortcuts(shortcuts: any) {
    await unregisterAll();
    
    await register(shortcuts.hideMeter.modifier + "+" + shortcuts.hideMeter.key, async () => {
        await invoke("toggle_meter_window");
    });
    await register(shortcuts.showLogs.modifier + "+" + shortcuts.showLogs.key, async () => {
        await invoke("open_url", { url: "logs" });
    });
    await register(shortcuts.showLatestEncounter.modifier + "+" + shortcuts.showLatestEncounter.key, async () => {
        await invoke("open_most_recent_encounter");
    });   
}

export const skillIcon = settingsStore("skillIcon", {});
export const classIconCache = settingsStore("classIconCache", {});