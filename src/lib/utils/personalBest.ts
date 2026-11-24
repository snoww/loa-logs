import { encounterMap, raidGates } from "#lib/constants/encounters.ts";
import { resolveLocalPlayerRole, supportBuffPerformance } from "#lib/constants/support.ts";
import type { Entity, PersonalBest, SupportBuffSummary } from "#lib/types.ts";

export const UNICODE_MINUS = "\u2212";
const DPS_CURVE_STEP_SECONDS = 5;
export const BRELSHAZA_G6_MAX_HP_THRESHOLD = 100_000_000_000;

const AMBIGUOUS_BRELSHAZA_BOSS = "Phantom Legion Commander Brelshaza";

const gateBossesByGate: Record<string, string[]> = {};
for (const raid of Object.values(encounterMap)) {
  for (const [gate, bosses] of Object.entries(raid)) {
    gateBossesByGate[gate] = bosses;
  }
}

export interface GateMatch {
  gateKey: string;
  bosses: string[];
  bossMaxHpMin?: number;
  bossMaxHpMax?: number;
}

export interface PersonalBestCacheKeyInput {
  localPlayer: string;
  gateKey: string;
  difficulty: string;
  isSupport: boolean;
  bossOnlyDamage: boolean;
}

export interface BossHpSample {
  time: number;
  hp: number;
  p: number;
}

export interface DpsPersonalBestComparison {
  kind: "dps";
  metric: "DPS" | "nDPS";
  liveDps: number;
  pbDps: number;
  deltaDps: number;
  deltaPercent: number;
  bossHpPercent: number;
  isAhead: boolean;
  logId: number;
  difficulty?: string;
  fightStart: number;
}

export interface SupportPersonalBestComparison {
  kind: "support";
  liveScore: number;
  pbScore: number;
  deltaScore: number;
  liveAp: number;
  liveBrand: number;
  liveIdentity: number;
  isAhead: boolean;
  logId: number;
  difficulty?: string;
  fightStart: number;
}

export type PersonalBestComparison = DpsPersonalBestComparison | SupportPersonalBestComparison;

// Keep synchronized with normalize_boss_name in src-tauri/src/utils.rs.
export function normalizeBossName(bossName: string): string {
  switch (bossName) {
    case "Chaos Lightning Dragon Jade":
      return "Argeos";
    case "Vicious Argeos":
    case "Ruthless Lakadroff":
    case "Untrue Crimson Yoho":
    case "Despicable Skolakia":
      return "Behemoth, the Storm Commander";
    case "Krathoios's Tail":
      return "Krathoios";
    default:
      return bossName;
  }
}

export function resolveGateMatch(bossName: string, bossMaxHp?: number): GateMatch | null {
  if (!bossName) return null;

  const normalized = normalizeBossName(bossName);
  let gate: string | undefined;
  let bossMaxHpMin: number | undefined;
  let bossMaxHpMax: number | undefined;

  if (normalized === AMBIGUOUS_BRELSHAZA_BOSS) {
    if (bossMaxHp === undefined || bossMaxHp <= 0) {
      return null;
    }
    if (bossMaxHp <= BRELSHAZA_G6_MAX_HP_THRESHOLD) {
      gate = "Brelshaza G6";
      bossMaxHpMax = BRELSHAZA_G6_MAX_HP_THRESHOLD;
    } else {
      gate = "Act 2: Brelshaza G2";
      bossMaxHpMin = BRELSHAZA_G6_MAX_HP_THRESHOLD + 1;
    }
  } else {
    gate = raidGates[normalized] ?? raidGates[bossName];
    if (gate === "Act 2: Brelshaza G2") {
      bossMaxHpMin = BRELSHAZA_G6_MAX_HP_THRESHOLD + 1;
    }
  }

  const baseBosses = gate && gateBossesByGate[gate] ? gateBossesByGate[gate] : [];
  const bosses = Array.from(new Set([...baseBosses, normalized, bossName])).sort();
  const match: GateMatch = {
    gateKey: gate && gateBossesByGate[gate] ? gate : normalized,
    bosses
  };
  if (bossMaxHpMin !== undefined) {
    match.bossMaxHpMin = bossMaxHpMin;
  }
  if (bossMaxHpMax !== undefined) {
    match.bossMaxHpMax = bossMaxHpMax;
  }
  return match;
}

export function buildPersonalBestCacheKey(input: PersonalBestCacheKeyInput): string {
  return JSON.stringify([input.localPlayer, input.gateKey, input.difficulty, input.isSupport, input.bossOnlyDamage]);
}

export interface PersonalBestLookupRequest<TCriteria> {
  key: string;
  criteria: TCriteria;
}

export class PersonalBestLookupController<TCriteria> {
  private readonly cache = new Map<string, PersonalBest | null>();
  private readonly fetchPersonalBest: (criteria: TCriteria) => Promise<PersonalBest | null>;
  private readonly onResult: (pb: PersonalBest | null) => void;
  private readonly onError: (err: unknown) => void;
  private activeKey = "";
  private requestId = 0;

  constructor(
    fetchPersonalBest: (criteria: TCriteria) => Promise<PersonalBest | null>,
    onResult: (pb: PersonalBest | null) => void,
    onError: (err: unknown) => void
  ) {
    this.fetchPersonalBest = fetchPersonalBest;
    this.onResult = onResult;
    this.onError = onError;
  }

  sync(req: PersonalBestLookupRequest<TCriteria> | null): void {
    if (!req) {
      this.activeKey = "";
      this.requestId++;
      this.onResult(null);
      return;
    }

    if (this.cache.has(req.key)) {
      this.activeKey = req.key;
      this.onResult(this.cache.get(req.key) ?? null);
      return;
    }

    if (this.activeKey === req.key) {
      return;
    }

    this.activeKey = req.key;
    this.onResult(null);
    const requestId = ++this.requestId;
    this.fetchPersonalBest(req.criteria)
      .then((pb) => {
        if (requestId !== this.requestId || this.activeKey !== req.key) {
          return;
        }
        this.cache.set(req.key, pb);
        this.onResult(pb);
      })
      .catch((err) => {
        if (requestId === this.requestId && this.activeKey === req.key) {
          this.onResult(null);
        }
        this.onError(err);
      });
  }

  invalidate(): void {
    this.cache.clear();
    this.activeKey = "";
    this.requestId++;
  }
}

export function getActiveFightDurationMs(
  durationMs: number,
  fightStart?: number,
  intermissionStart?: number,
  intermissionEnd?: number
): number {
  if (durationMs <= 0) {
    return 0;
  }
  if (fightStart === undefined || fightStart <= 0 || intermissionStart === undefined) {
    return durationMs;
  }
  const nowMs = fightStart + durationMs;
  const pauseStart = Math.max(intermissionStart, fightStart);
  const pauseEnd = intermissionEnd !== undefined ? Math.min(nowMs, intermissionEnd) : nowMs;
  const pausedMs = pauseEnd > pauseStart ? pauseEnd - pauseStart : 0;
  return Math.max(durationMs - pausedMs, 0);
}

export function findBossHpTime(bossHpLog: ReadonlyArray<BossHpSample>, targetHpRatio: number): number | null {
  if (bossHpLog.length === 0 || !Number.isFinite(targetHpRatio)) {
    return null;
  }
  if (targetHpRatio === bossHpLog[0].p) {
    return bossHpLog[0].time;
  }
  // At pull start (t = 0), the first logged sample is recorded after the opening hit
  // so its p is slightly below 1.0 even though the boss started at 100% HP at t = 0.
  if (bossHpLog[0].time === 0 && targetHpRatio > bossHpLog[0].p && targetHpRatio <= 1) {
    return 0;
  }
  for (let i = 0; i < bossHpLog.length - 1; i++) {
    const a = bossHpLog[i];
    const b = bossHpLog[i + 1];
    if (a.p >= targetHpRatio && b.p <= targetHpRatio) {
      if (a.p === b.p) {
        return a.time;
      }
      const frac = (a.p - targetHpRatio) / (a.p - b.p);
      return a.time + frac * (b.time - a.time);
    }
  }
  return null;
}

export function interpolateCurveAtTime(
  curve: ReadonlyArray<number>,
  timeSeconds: number,
  stepSeconds = DPS_CURVE_STEP_SECONDS,
  curveEndSeconds?: number
): number | null {
  if (curve.length === 0 || !Number.isFinite(timeSeconds) || stepSeconds <= 0) {
    return null;
  }
  if (timeSeconds <= 0 || curve.length === 1) {
    return curve[0];
  }

  const hasTailSample =
    curveEndSeconds !== undefined &&
    Number.isFinite(curveEndSeconds) &&
    curve.length >= 2 &&
    curveEndSeconds > (curve.length - 2) * stepSeconds &&
    curveEndSeconds < (curve.length - 1) * stepSeconds;

  if (hasTailSample) {
    const lastRegularIndex = curve.length - 2;
    const lastRegularTime = lastRegularIndex * stepSeconds;
    if (timeSeconds >= curveEndSeconds) {
      return curve[curve.length - 1];
    }
    if (timeSeconds > lastRegularTime) {
      const frac = (timeSeconds - lastRegularTime) / (curveEndSeconds - lastRegularTime);
      return curve[lastRegularIndex] + frac * (curve[curve.length - 1] - curve[lastRegularIndex]);
    }
  }

  const maxRegularIndex = hasTailSample ? curve.length - 2 : curve.length - 1;
  const position = timeSeconds / stepSeconds;
  if (position >= maxRegularIndex) {
    return curve[maxRegularIndex];
  }
  const index = Math.floor(position);
  const frac = position - index;
  return curve[index] + frac * (curve[index + 1] - curve[index]);
}

export function evaluatePersonalBest(input: {
  pb: PersonalBest;
  entity: Pick<Entity, "name" | "classId" | "spec">;
  damageDealt: number;
  baseDamage: number;
  durationMs: number;
  fightStart?: number;
  intermissionStart?: number;
  intermissionEnd?: number;
  rdpsValid?: boolean;
  currentBoss: Pick<Entity, "name" | "currentHp" | "maxHp" | "currentShield" | "isDead"> | null;
  localSupportBuffs?: SupportBuffSummary | null;
}): PersonalBestComparison | null {
  const {
    pb,
    entity,
    damageDealt,
    baseDamage,
    durationMs,
    fightStart,
    intermissionStart,
    intermissionEnd,
    rdpsValid,
    currentBoss,
    localSupportBuffs
  } = input;

  const liveRole = resolveLocalPlayerRole(entity);
  if (!liveRole || pb.isSupport !== (liveRole === "support")) {
    return null;
  }

  if (pb.isSupport) {
    if (pb.buffPerformance === undefined || pb.buffPerformance <= 0 || !localSupportBuffs) {
      return null;
    }
    const liveScore =
      supportBuffPerformance(localSupportBuffs.ap, localSupportBuffs.brand, localSupportBuffs.identity) * 100;
    const pbScore = pb.buffPerformance * 100;
    const deltaScore = liveScore - pbScore;
    return {
      kind: "support",
      liveScore,
      pbScore,
      deltaScore,
      liveAp: localSupportBuffs.ap * 100,
      liveBrand: localSupportBuffs.brand * 100,
      liveIdentity: localSupportBuffs.identity * 100,
      isAhead: deltaScore >= 0,
      logId: pb.id,
      difficulty: pb.difficulty,
      fightStart: pb.fightStart
    };
  }

  const useNdps = !!pb.ndpsAverage?.length;
  if (useNdps && rdpsValid === false) {
    return null;
  }

  const activeDurationMs = getActiveFightDurationMs(durationMs, fightStart, intermissionStart, intermissionEnd);
  if (activeDurationMs <= 0 || !currentBoss || currentBoss.maxHp <= 0) {
    return null;
  }

  const curve = useNdps ? pb.ndpsAverage : pb.dpsAverage;
  if (!curve || curve.length === 0) {
    return null;
  }
  const bossHpLog = pb.bossHpLog?.[currentBoss.name];
  if (!bossHpLog || bossHpLog.length === 0) {
    return null;
  }

  const effectiveHp =
    currentBoss.isDead || currentBoss.currentHp < 0 ? 0 : currentBoss.currentHp + (currentBoss.currentShield ?? 0);
  const liveHpRatio = effectiveHp / currentBoss.maxHp;
  const pbTime = findBossHpTime(bossHpLog, liveHpRatio);
  if (pbTime === null) {
    return null;
  }
  const pbDps = interpolateCurveAtTime(curve, pbTime, DPS_CURVE_STEP_SECONDS, pb.curveEndSeconds);
  if (pbDps === null || pbDps <= 0) {
    return null;
  }

  const liveDps = Math.round((useNdps ? baseDamage : damageDealt) / (activeDurationMs / 1000));
  const deltaDps = liveDps - pbDps;
  const deltaPercent = (deltaDps / pbDps) * 100;

  return {
    kind: "dps",
    metric: useNdps ? "nDPS" : "DPS",
    liveDps,
    pbDps,
    deltaDps,
    deltaPercent,
    bossHpPercent: liveHpRatio * 100,
    isAhead: deltaPercent >= 0,
    logId: pb.id,
    difficulty: pb.difficulty,
    fightStart: pb.fightStart
  };
}

export function formatSignedFixed(value: number, digits = 1, suffix = ""): string {
  const sign = value < 0 ? UNICODE_MINUS : "+";
  return `${sign}${Math.abs(value).toFixed(digits)}${suffix}`;
}

export function formatSignedAbbreviated(value: number, abbreviate: (n: number) => string): string {
  const sign = value < 0 ? UNICODE_MINUS : "+";
  return `${sign}${abbreviate(Math.abs(value))}`;
}

export function formatShortPbDate(timestampMs: number, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { month: "short", day: "numeric" }).format(new Date(timestampMs));
}

export function formatPbLogMeta(
  logId: number,
  difficulty: string | undefined,
  fightStart: number,
  formatDate: (timestampMs: number) => string = formatShortPbDate
): string {
  const parts = [`PB #${logId}`];
  if (difficulty) {
    parts.push(difficulty);
  }
  if (fightStart > 0) {
    parts.push(formatDate(fightStart));
  }
  return parts.join(" \u00b7 ");
}
