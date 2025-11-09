export interface GetStatsArgs {
  dateFrom: string;
  dateTo: string;
}

export interface GetStatsResponse {
  items: RaidStats[];
}

export interface RaidStats {
  name: string;
  order: number;
  raidType: string;
  count: number;
  dps: DpsUnit | null;
  uptimes: [string, string, string, string];
  instances: RaidMetric[];
  isFinalGate: boolean;
  isGuardianRaid: boolean;
}

export interface RaidMetric {
  kind: string;
  playedAsSupport: boolean;
  dps: number;
  supportAp: number;
  supportBrand: number;
  supportIdentity: number;
  supportHyper: number;
}

export interface DpsUnit {
  formatted: string;
  raw: number;
}