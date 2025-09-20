export type FontScale = "0" | "1" | "2" | "3";

export interface DisplayFlags {
  damage: boolean;
  dps: boolean;
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
  positionalDmgPercent?: boolean;
  stagger?: boolean;
}

export interface BreakdownBase extends DisplayFlags {
  avgDamage: boolean;
  maxDamage: boolean;
  casts: boolean;
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
  bossOnlyDamageDefaultOn: boolean;
  startOnBoot: boolean;
  logsPerPage: number;
  experimentalFeatures: boolean;
  mini: boolean;
  miniEdit: boolean;
  autoShow: boolean;
  autoHideDelay: number;
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

export interface MeterSettings extends DisplayFlags {
  bossInfo: boolean;
  bossHpBar: boolean;
  splitBossHpBar: boolean;
  showTimeUntilKill: boolean;
  splitPartyBuffs: boolean;
  showClassColors: boolean;
  profileShortcut: boolean;
  pinSelfParty: boolean;
  breakdown: MeterBreakdown;
}

export interface LogsSettings extends DisplayFlags {
  abbreviateHeader: boolean;
  splitPartyDamage: boolean;
  splitPartyBuffs: boolean;
  profileShortcut: boolean;
  minEncounterDuration: number;
  breakdown: LogsBreakdown;
}

export interface MiniSettings {
  info: string;
  bossHpBar: boolean;
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
