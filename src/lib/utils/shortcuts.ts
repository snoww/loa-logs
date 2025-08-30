import { misc, settings } from "$lib/stores.svelte";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { register } from "@tauri-apps/plugin-global-shortcut";

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
    name: "Toggle Clickthrough",
    action: async () => {
      await invoke("set_clickthrough", { set: !misc.clickthrough });
      misc.clickthrough = !misc.clickthrough;
    }
  }
};

export async function registerShortcuts() {
  if (misc.modifyingShortcuts) return;
  try {
    for (const sc of Object.entries(shortcuts)) {
      const shortcut = settings.app.shortcuts[sc[0] as keyof typeof settings.app.shortcuts];
      if (shortcut) {
        await register(shortcut, (event) => {
          if (event.state === "Pressed") {
            sc[1].action();
          }
        });
      }
    }
  } catch {}
}
