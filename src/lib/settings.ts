export type FontScale = "0" | "1" | "2" | "3";

export interface DisplayFlags {
  damage: boolean;
  ndmg: boolean;
  rdmg: boolean;
  dps: boolean;
  ndps: boolean;
  rdps: boolean;
  unbuffedDamage: boolean;
  unbuffedDps: boolean;
  damagePercent: boolean;
  critRate: boolean;
  critDmg: boolean;
  frontAtk: boolean;
  backAtk: boolean;
  percentBuffBySup: boolean;
  percentIdentityBySup: boolean;
  percentBrand: boolean;
  percentHatBySup: boolean;
  supportContrib: boolean;
  rdpsContrib: boolean;
}

export interface BreakdownBase extends DisplayFlags {
  avgDamage: boolean;
  maxDamage: boolean;
  casts: boolean;
  interrupts: boolean;
  cpm: boolean;
  hits: boolean;
  hpm: boolean;
}

export interface MeterBreakdown extends BreakdownBase {}

export interface LogsBreakdown extends BreakdownBase {
  adjustedCritRate: boolean;
}

export interface GeneralSettings {
  startLoaOnStart: boolean;
  lowPerformanceMode: boolean;
  showNames: boolean;
  showGearScore: boolean;
  hideNames: boolean;
  showEsther: boolean;
  hideLogo: boolean;
  showDate: boolean;
  showDifficulty: boolean;
  showGate: boolean;
  showDetails: boolean;
  showShields: boolean;
  showTanked: boolean;
  showBosses: boolean;
  showRaidsOnly: boolean;
  splitLines: boolean;
  underlineHovered: boolean;
  accentColor: string;
  autoIface: boolean;
  port: number;
  blur: boolean;
  blurWin11: boolean;
  isWin11: boolean;
  transparent: boolean;
  scale: FontScale;
  logScale: FontScale;
  alwaysOnTop: boolean;
  bossOnlyDamage: boolean;
  keepFavorites: boolean;
  hideMeterOnStart: boolean;
  hideLogsOnStart: boolean;
  constantLocalPlayerColor: boolean;
  startOnBoot: boolean;
  logsPerPage: number;
  experimentalFeatures: boolean;
  mini: boolean;
  miniEdit: boolean;
  autoShow: boolean;
  autoHideDelay: number;
  betaChannel: boolean;
  exitlagCompat: boolean;
}

export interface Shortcuts {
  hideMeter: string;
  showLogs: string;
  showLatestEncounter: string;
  resetSession: string;
  pauseSession: string;
  manualSave: string;
  disableClickthrough: string;
}

export interface CommonSettings extends DisplayFlags {
  splitPartyBuffs: boolean;
  profileShortcut: boolean;
  incapacitatedTime: boolean;
  deathTime: boolean;
  counters: boolean;
  skillInterrupts: boolean;
  positionalDmgPercent: boolean;
  stagger: boolean;
}

export interface MeterSettings extends CommonSettings {
  bossInfo: boolean;
  bossHpBar: boolean;
  splitBossHpBar: boolean;
  showTimeUntilKill: boolean;
  showClassColors: boolean;
  pinSelfParty: boolean;
  breakdown: MeterBreakdown;
}

export interface LogsSettings extends CommonSettings {
  abbreviateHeader: boolean;
  splitPartyDamage: boolean;
  minEncounterDuration: number;
  breakdown: LogsBreakdown;
}

export interface MiniSettings {
  info: string;
  bossHpBar: boolean;
  previewPlayers: 4 | 8;
}

export interface BuffSettings {
  default: boolean;
}

export interface AppSettings {
  general: GeneralSettings;
  shortcuts: Shortcuts;
  meter: MeterSettings;
  logs: LogsSettings;
  mini: MiniSettings;
  buffs: BuffSettings;
}

export const defaultSettings: AppSettings = {
  general: {
    startLoaOnStart: false,
    lowPerformanceMode: false,
    showNames: true,
    showGearScore: true,
    hideNames: false,
    showEsther: true,
    hideLogo: false,
    showDate: true,
    showDifficulty: true,
    showGate: false,
    showDetails: false,
    showShields: true,
    showTanked: false,
    showBosses: false,
    showRaidsOnly: true,
    splitLines: true,
    underlineHovered: false,
    accentColor: "theme-violet",
    autoIface: true,
    port: 6040,
    blur: true,
    blurWin11: false,
    isWin11: false,
    transparent: true,
    scale: "1",
    logScale: "1",
    alwaysOnTop: true,
    bossOnlyDamage: true,
    keepFavorites: true,
    hideLogsOnStart: false,
    constantLocalPlayerColor: false,
    startOnBoot: false,
    hideMeterOnStart: false,
    logsPerPage: 10,
    experimentalFeatures: false,
    mini: false,
    miniEdit: true,
    autoShow: false,
    autoHideDelay: 5,
    betaChannel: false,
    exitlagCompat: false
  },
  shortcuts: {
    hideMeter: "Control+ArrowDown",
    showLogs: "Control+ArrowUp",
    showLatestEncounter: "",
    resetSession: "",
    pauseSession: "",
    manualSave: "",
    disableClickthrough: ""
  },
  meter: {
    bossInfo: true,
    bossHpBar: true,
    splitBossHpBar: false,
    showTimeUntilKill: false,
    splitPartyBuffs: true,
    showClassColors: true,
    profileShortcut: false,
    damage: false,
    ndmg: false,
    rdmg: false,
    dps: true,
    ndps: false,
    rdps: false,
    unbuffedDamage: false,
    unbuffedDps: false,
    damagePercent: true,
    deathTime: false,
    incapacitatedTime: false,
    critRate: true,
    critDmg: false,
    frontAtk: true,
    backAtk: true,
    counters: false,
    skillInterrupts: false,
    pinSelfParty: false,
    positionalDmgPercent: true,
    percentBuffBySup: true,
    percentIdentityBySup: true,
    percentBrand: true,
    percentHatBySup: true,
    supportContrib: false,
    rdpsContrib: false,
    stagger: false,
    breakdown: {
      damage: true,
      ndmg: false,
      rdmg: false,
      dps: true,
      ndps: false,
      rdps: false,
      unbuffedDamage: false,
      unbuffedDps: false,
      damagePercent: true,
      critRate: true,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: false,
      maxDamage: true,
      casts: true,
      interrupts: false,
      cpm: true,
      hits: false,
      hpm: false,
      percentBuffBySup: false,
      percentIdentityBySup: false,
      percentBrand: false,
      percentHatBySup: false,
      supportContrib: false,
      rdpsContrib: false
    }
  },
  mini: {
    info: "damage",
    bossHpBar: false,
    previewPlayers: 8
  },
  logs: {
    abbreviateHeader: false,
    splitPartyDamage: true,
    splitPartyBuffs: true,
    profileShortcut: true,
    damage: true,
    ndmg: false,
    rdmg: false,
    dps: true,
    ndps: true,
    rdps: true,
    unbuffedDamage: false,
    unbuffedDps: false,
    damagePercent: true,
    deathTime: true,
    incapacitatedTime: true,
    critRate: true,
    critDmg: false,
    frontAtk: true,
    backAtk: true,
    counters: true,
    skillInterrupts: true,
    minEncounterDuration: 30,
    positionalDmgPercent: true,
    percentBuffBySup: true,
    percentIdentityBySup: true,
    percentHatBySup: true,
    percentBrand: true,
    supportContrib: false,
    rdpsContrib: true,
    stagger: true,
    breakdown: {
      damage: true,
      ndmg: false,
      rdmg: false,
      dps: true,
      ndps: true,
      rdps: false,
      unbuffedDamage: false,
      unbuffedDps: false,
      damagePercent: true,
      critRate: true,
      adjustedCritRate: true,
      critDmg: false,
      frontAtk: true,
      backAtk: true,
      avgDamage: true,
      maxDamage: true,
      casts: true,
      interrupts: true,
      cpm: true,
      hits: true,
      hpm: true,
      percentBuffBySup: false,
      percentIdentityBySup: false,
      percentBrand: false,
      percentHatBySup: false,
      supportContrib: false,
      rdpsContrib: false
    }
  },
  buffs: {
    default: true
  }
};

/**
 * Merge saved settings into a target defaults object in place.
 */
export const mergeSettings = (target: any, storageSettings: any) => {
  if (!storageSettings || typeof storageSettings !== "object") return;
  for (const key of Object.keys(storageSettings)) {
    if (key in target) {
      const value = storageSettings[key];
      if (value === undefined) continue;
      if (typeof value === "object" && value !== null && typeof target[key] === "object" && target[key] !== null) {
        mergeSettings(target[key], value);
      } else if (value !== null) {
        target[key] = value;
      }
    }
  }
};

export function mergeWithDefaultSettings(saved?: Record<string, unknown> | AppSettings | null): AppSettings {
  const merged = structuredClone(defaultSettings);
  if (saved) {
    mergeSettings(merged, saved);
  }
  return merged;
}
