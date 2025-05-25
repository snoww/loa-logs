import { browser } from "$app/environment";
import { defaultClassColors, defaultSettings, type LogSettings } from "$lib/utils/settings";
import { invoke } from "@tauri-apps/api";
import { join, resourceDir } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/tauri";

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
  appSettings = $state(defaultSettings);
  classColors = $state<Record<string, string>>(defaultClassColors);
  imagePath = $state("");
  iconPath = $state("");
  classIconPath = $state("");
  lockUpdate = false;

  constructor() {
    if (!browser) return;

    (async () => {
      this.imagePath = convertFileSrc(await join(await resourceDir(), "images"));
      this.iconPath = convertFileSrc(await join(await resourceDir(), "images", "skills"));
      this.classIconPath = convertFileSrc(await join(await resourceDir(), "images", "classes"));
    })();
    if (localStorage) {
      const updateSettings = (settings: string | null) => {
        this.lockUpdate = true;
        if (settings) {
          try {
            const settingsFromStorage = JSON.parse(settings) as LogSettings;
            mergeSettings(defaultSettings, settingsFromStorage);
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

      updateSettings(localStorage.getItem("appSettings"));
      updateClassColors(localStorage.getItem("classColors"));

      $effect.root(() => {
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("appSettings", JSON.stringify(this.appSettings));
          invoke("save_settings", { settings: this.appSettings });
        });
        $effect(() => {
          if (this.lockUpdate) return;
          localStorage.setItem("classColors", JSON.stringify(this.classColors));
        });
      });

      window.addEventListener("storage", (e) => {
        if (this.lockUpdate) return;
        const { key, newValue, storageArea } = e;
        if (storageArea !== localStorage) return;
        if (key === "appSettings") updateSettings(newValue);
        else if (key === "classColors") updateClassColors(newValue);
        else return;
      });
    } else {
      console.warn("localStorage not available?");
    }
  }
}

export const settings = new Settings();
