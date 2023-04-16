import { invoke } from '@tauri-apps/api';
import { register, unregisterAll } from '@tauri-apps/api/globalShortcut';
import { writable } from 'svelte/store';

export const defaultSettings = {
    "general": {
        "showNames": true,
    },
    "shortcuts": {
        "hideMeter": {
            "modifier": "Ctrl",
            "key": "ArrowDown",
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

const settingsStore = (key: string) => {
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
            store.set(value);
        },
        update: store.update
    };
};

export const settings = settingsStore("settings");

export async function registerShortcut(modifier: string, key: string) {
    await unregisterAll();
    const shortcut = modifier + '+' + key;
    await register(shortcut, async () => {
        await invoke("toggle_meter_window");
    });
}
