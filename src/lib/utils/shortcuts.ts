import { manualSave, openMostRecentEncounter, pauseSession, resetSession, setClickthrough, toggleLogsWindow, toggleMeterWindow } from "$lib/api";
import { misc, settings } from "$lib/stores.svelte";
import { register } from "@tauri-apps/plugin-global-shortcut";

export type Shortcut = {
  name: string;
  action: () => void | Promise<void>;
};

export const shortcuts: Record<string, Shortcut> = {
  hideMeter: {
    name: "Hide Meter",
    action: async () => await toggleMeterWindow()
  },
  showLogs: {
    name: "Show Logs",
    action: async () => await toggleLogsWindow()
  },
  showLatestEncounter: {
    name: "Show Latest Encounter",
    action: async () => await openMostRecentEncounter()
  },
  resetSession: {
    name: "Reset Session",
    action: async () => await resetSession()
  },
  pauseSession: {
    name: "Pause Session",
    action: async () => await pauseSession()
  },
  manualSave: {
    name: "Manual Save",
    action: async () => await manualSave()
  },
  disableClickthrough: {
    name: "Toggle Clickthrough",
    action: async () => {
      await setClickthrough(!misc.clickthrough);
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
        await register(shortcut, () => {
          sc[1].action();
        });
      }
    }
  } catch {}
}
