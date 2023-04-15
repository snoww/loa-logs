import { writable } from 'svelte/store';

export const defaultSettings = {
    "general": {
    },
    "shortcuts": {
        "hideMeter": {
            "value": "CommandOrControl+Down",
            "default": "CommandOrControl+Down"
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

export const settingsStore = (key: string, initial: object) => {
    if (localStorage.getItem(key) === null) {
        localStorage.setItem(key, JSON.stringify(initial));
    }
    const saved = JSON.parse(localStorage.getItem(key) || JSON.stringify(defaultSettings));

    const { subscribe, set, update } = writable(saved);
    return {
        subscribe,
        set: (value: object) => {
            localStorage.setItem(key, JSON.stringify(value));
            return set(value);
        },
        update
    };
};

export const settings = settingsStore("settings", defaultSettings);
