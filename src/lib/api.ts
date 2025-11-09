import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen, type EventCallback, type UnlistenFn } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import type {
  Encounter,
  EncounterDbInfo,
  EncounterEvent,
  EncountersOverview,
  IdentityEvent,
  PartyEvent
} from "./types";
import type { AppSettings } from "./settings";
import type { GetStatsArgs, GetStatsResponse } from "./stats";

export const getAppVersion = async (): Promise<string> => `v${await getVersion()}`;

export const openUrl = (url: string): Promise<void> => invoke("open_url", { url });

export enum Window {
  Logs,
  Meter
}

export const toggleWindow = (window: Window): Promise<void> => {
  if (window === Window.Logs) {
    return invoke("toggle_logs_window");
  }

  return invoke("toggle_meter_window");
};

export const getRaidStats = (args: GetStatsArgs): Promise<GetStatsResponse> => invoke("get_raid_stats", { args });

export const openMostRecentEncounter = (): Promise<void> => invoke("open_most_recent_encounter");

export const checkStartOnBoot = (): Promise<boolean> => invoke("check_start_on_boot");

export const checkLoaRunning = (): Promise<boolean> => invoke("check_loa_running");

export const toggleEncounterFavorite = (id: number): Promise<boolean> => invoke("toggle_encounter_favorite", { id });

export const setClickthrough = (set: boolean): Promise<void> => invoke("set_clickthrough", { set });

export const saveSettings = (settings: AppSettings): Promise<void> => invoke("save_settings", { settings });

export const getSettings = (): Promise<AppSettings> => invoke("get_settings");

export const getDbInfo = (minDuration: number): Promise<EncounterDbInfo> => invoke("get_db_info", { minDuration });

export const openDbPath = (): Promise<void> => invoke("open_db_path");

export const setStartOnBoot = (set: boolean): Promise<void> => invoke("set_start_on_boot", { set });

export const setAlwaysOnTop = (enabled: boolean): Promise<void> => {
  if (enabled) {
    return invoke("enable_aot");
  }

  return invoke("disable_aot");
};

export const writeLog = (message: string): Promise<void> => invoke("write_log", { message });

export const optimizeDatabase = (): Promise<void> => invoke("optimize_database");

export const startLoaProcess = (): Promise<void> => invoke("start_loa_process");

interface LoadEncountersCriteria {
  page: number;
  pageSize: number;
  search: string;
  filter: {
    minDuration?: number;
    bosses?: string[];
    cleared?: boolean;
    favorite?: boolean;
    difficulty?: string;
    sort?: string;
    order?: "asc" | "desc";
    raidsOnly?: boolean;
  };
}

export const loadEncountersPreview = (criteria: LoadEncountersCriteria): Promise<EncountersOverview> =>
  invoke("load_encounters_preview", { ...criteria });

export interface SyncArgs {
  encounter: number;
  upstream: string;
  failed: boolean;
}

export const sync = (args: SyncArgs): Promise<void> => invoke("sync", { ...args });

export type DeleteEncountersArgs =
  | {
      type: "all";
      keepFavorites: boolean;
    }
  | {
      type: "ids";
      ids: number[];
    }
  | {
      type: "uncleared";
      keepFavorites: boolean;
    }
  | {
      type: "duration";
      keepFavorites: boolean;
      minDuration: number;
    };

export const deleteEncounters = (args: DeleteEncountersArgs): Promise<void> => {
  switch (args.type) {
    case "all":
      return invoke("delete_all_encounters", args);
    case "ids":
      return invoke("delete_encounters", args);
    case "uncleared":
      return invoke("delete_all_uncleared_encounters", args);
    case "duration":
      return invoke("delete_encounters_below_min_duration", args);
  }

  return Promise.resolve();
};

export const deleteEncounter = (id: string): Promise<void> => invoke("delete_encounter", { id });

export const setBlur = (enabled: boolean): Promise<void> => {
  if (enabled) {
    return invoke("enable_blur");
  }

  return invoke("disable_blur");
};

export const relaunchApp = async () => {
  await invoke("unload_driver");
  await invoke("remove_driver");
  await relaunch();
};

export const getSyncCandidates = (forceResync: boolean): Promise<number[]> =>
  invoke("get_sync_candidates", { forceResync });

export const loadEncounter = (id: string): Promise<Encounter> => invoke("load_encounter", { id });

export const pauseRequest = (): Promise<void> => emit("pause-request");

export const saveRequest = (): Promise<void> => emit("save-request");

export const resetRequest = (): Promise<void> => emit("reset-request");

export const emitDetailsRequest = (): Promise<void> => emit("emit-details-request");

export const setBossOnlyDamage = (enabled: boolean): Promise<void> => emit("boss-only-damage-request", enabled);

export const onLatestEncounter = (handler: (event: any) => void) => listen("show-latest-encounter", handler);

export const onRedirectUrl = (handler: (event: any) => void) => listen("redirect-url", handler);

export const onIdentityUpdate = (handler: (event: IdentityEvent) => void) =>
  listen<IdentityEvent>("identity-update", (event) =>
    handler({
      ...event.payload,
      timestamp: +Date.now()
    })
  );

export const onEncounterUpdate = (handler: (event: EncounterEvent) => void) => listen("encounter-update", handler);

export const onPartyUpdate = (handler: (event: PartyEvent) => void) => listen("party-update", handler);

export const onInvalidDamage = (handler: () => void) => listen("invalid-damage", handler);

export const onZoneChange = (handler: () => void) => listen("zone-change", handler);

export const onRaidStart = (handler: () => void) => listen("raid-start", handler);

export const onResetEncounter = (handler: () => void) => listen("reset-encounter", handler);

export const onPauseEncounter = (handler: () => void) => listen("pause-encounter", handler);

export const onSaveEncounter = (handler: () => void) => listen("save-encounter", handler);

export const onPhaseTransition = (handler: (event: { payload: number }) => void) => listen("phase-transition", handler);

export const onAdmin = (handler: () => void) => listen("admin", handler);

export const onClearEncounter = (handler: (event: { payload: number }) => void) => listen("clear-encounter", handler);
