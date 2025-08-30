import { invoke } from "@tauri-apps/api/core";
import type { Encounter, EncounterDbInfo, EncountersOverview, PartyInfo } from "./types";
import type { LogSettings } from "./stores.svelte";
import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { check } from '@tauri-apps/plugin-updater';

const callbacks: Array<UnlistenFn> = new Array();

export interface SyncArgs extends Record<string, unknown> {
    encounter: number;
    upstream: string;
    failed: boolean;
}

export interface DeleteEncounterArgs extends Record<string, unknown> {
  minDuration: number;
  keepFavorites: boolean;
}

export interface LoadEncountersOverviewArgs extends Record<string, unknown> {
  page: number;
  pageSize: number;
  search: string;
  filter: {
    minDuration: number;
    bosses: string[];
    cleared: boolean;
    favorite: boolean;
    difficulty: string;
    sort: string;
    order: string;
    raidsOnly: boolean;
  }
}

export const checkUpdate = async () => {
  try {
    let update = await check();
  
    if(update?.available) {
      return {
        shouldUpdate: true,
        manifest: {
          body: update.body || ""
        }
      }
    }

    return {
      shouldUpdate: false,
    }
  } catch (error) {
    return {
      error,
      shouldUpdate: false,
    }
  }
}

export const getDbInfo = (minDuration: number): Promise<EncounterDbInfo> => invoke<EncounterDbInfo>("get_db_info", { minDuration });

export const deleteAllUnclearedEncounters = (keepFavorites: boolean): Promise<void> => 
  invoke("delete_all_uncleared_encounters", { keepFavorites });

export const deleteEncountersBelowMinDuration = (args: DeleteEncounterArgs): Promise<void> => 
  invoke("delete_encounters_below_min_duration", args);

export const deleteAllEncounters = (keepFavorites: boolean): Promise<void> => 
  invoke("delete_all_encounters", { keepFavorites });

export const deleteEncounter = (id: string | undefined): Promise<void> => invoke("delete_encounter", { id });

export const deleteEncounters = (ids: number[]): Promise<void> => invoke("delete_encounters", { ids });

export const getMiniWindow = (): Promise<WebviewWindow | null> => WebviewWindow.getByLabel("mini");

export const getMainWindow = (): Promise<WebviewWindow | null> => WebviewWindow.getByLabel("main");

export const loadEncounter = (id: string): Promise<Encounter> => invoke<Encounter>("load_encounter", { id });

export const loadEncountersPreview = (args: LoadEncountersOverviewArgs): Promise<EncountersOverview> => 
  invoke<EncountersOverview>("load_encounters_preview", args);

export const saveSettings = (settings: any): Promise<void> => invoke("save_settings", { settings });

export const getSettings = (): Promise<LogSettings> => invoke<LogSettings>("get_settings");

export const enableBlur = (): Promise<void> => invoke("enable_blur");

export const disableBlur = (): Promise<void> => invoke("disable_blur");

export const toggleEncounterFavorite = (id: number): Promise<void> => invoke("toggle_encounter_favorite", { id });

export const optimizeDatabase = (): Promise<void> => invoke("optimize_database");

export const unloadDriver = (): Promise<void> => invoke("unload_driver");

export const removeDriver = (): Promise<void> => invoke("remove_driver");

export const toggleMeterWindow = (): Promise<void> => invoke("toggle_meter_window");

export const toggleLogsWindow = (): Promise<void> => invoke("toggle_logs_window");

export const openMostRecentEncounter = (): Promise<void> => invoke("open_most_recent_encounter");

export const resetSession = (): Promise<void> => emit("reset-request");

export const pauseSession = (): Promise<void> => emit("pause-request");

export const manualSave = (): Promise<void> => emit("save-request");

export const setClickthrough = (set: boolean): Promise<void> => invoke("set_clickthrough", { set });

export const enableAot = (): Promise<void> => invoke("enable_aot");

export const disableAot = (): Promise<void> => invoke("disable_aot");

export const checkStartOnBoot = (): Promise<boolean> => invoke<boolean>("check_start_on_boot");

export const setStartOnBoot = (set: boolean): Promise<void> => invoke("set_start_on_boot", { set });

export const getSyncCandidates = (forceResync: boolean): Promise<number[]> =>
  invoke<number[]>("get_sync_candidates", { forceResync });

export const openDbPath = (): Promise<void> => invoke("open_db_path");

export const openUrl = (url: string): Promise<void> => invoke("open_url", { url });

export const sync = (args: SyncArgs): Promise<void> => invoke("sync", args);

export const checkLoaRunning = (): Promise<boolean> => invoke<boolean>("check_loa_running");

export const startLoaProcess = (): Promise<boolean> => invoke<boolean>("start_loa_process");

export const onEncounterUpdate = (handler: (encounter: Encounter) => void): Promise<UnlistenFn> =>
    listen<Encounter>("encounter-update", (event) => {
        handler(event.payload);
});
      
export const onPartyUpdate = async (handler: (value: PartyInfo | undefined) => void): Promise<void> => {
  const callback = await listen<PartyInfo | undefined>("party-update", (event) => {
    handler(event.payload);
  });

  callbacks.push(callback);
}

export const onInvalidDamage = async (handler: () => void): Promise<void> => {
  const callback = await listen("invalid-damage", (event) => {
    handler();
  });

  callbacks.push(callback);
}

export const onZoneChange = async (handler: () => void): Promise<void> => {
  const callback = await listen("zone-change", (event) => {
    handler();
  });

  callbacks.push(callback);
}

export const onRaidStart = async (handler: () => void): Promise<void> => {
  const callback = await listen("raid-start", (event) => {
    handler();
  });

  callbacks.push(callback);
}
    
export const onResetEncounter = async (handler: () => void): Promise<void> => {
  const callback = await listen("reset-encounter", (event) => {
    handler();
  });

  callbacks.push(callback);
}

export const onPauseEncounter = async (handler: () => void): Promise<void> => {
  const callback = await listen("pause-encounter", (event) => {
    handler();
  });
}

export const onSaveEncounter = async (handler: () => void): Promise<void> => {
  const callback = await listen("save-encounter", (event) => {
    handler();
  });

  callbacks.push(callback);
}

export const onPhaseTransition = async (handler: (phaseCode: number) => void): Promise<void> => {
  const callback = await listen<number>("phase-transition", (event) => {
    handler(event.payload);
  });

  callbacks.push(callback);
}

export const onAdmin = async (handler: () => void): Promise<void> => {
  const callback = await listen("admin", (event) => {
    handler();
  });

  callbacks.push(callback);
}

export const onClearEncounter = async (handler: (id: string) => void): Promise<void> => {
  const callback = await listen<number>("clear-encounter", (event) => {
    handler(event.payload.toString());
  });

  callbacks.push(callback);
}

export const unregisterAll = () => {
  for(const callback of callbacks) {
    callback();
  }

  callbacks.length = 0;
}