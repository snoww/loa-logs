import { misc, settings } from "$lib/stores.svelte";
import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { register } from "@tauri-apps/api/globalShortcut";
import { get } from "svelte/store";

export const UWUOWO_URL = "https://uwuowo.mathi.moe";

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
      await invoke("set_clickthrough", { set: false });
    }
  }
};

export async function registerShortcuts() {
  try {
    for (const sc of Object.entries(shortcuts)) {
      const shortcut = settings.app.shortcuts[sc[0] as keyof typeof settings.app.shortcuts];
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
