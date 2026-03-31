import { abbreviateNumberSplit, customRound } from "$lib/utils";
import type { EntityState } from "./entity.svelte";
import { settings } from "./stores.svelte";
import type { Skill } from "./types";
import type { WindowedSkillStats } from "./utils/windowedStats";

export class SkillState {
  skill: Skill = $state()!;
  entity: EntityState = $state()!;

  wss: WindowedSkillStats | undefined = $derived(this.entity.ws?.skillStats.get(this.skill.id));

  // windowed getters
  totalDamage = $derived(this.wss?.totalDamage ?? this.skill.totalDamage);
  hits = $derived(this.wss?.hits ?? this.skill.hits);
  crits = $derived(this.wss?.crits ?? this.skill.crits);
  critDmg = $derived(this.wss?.critDamage ?? this.skill.critDamage);
  ba = $derived(this.wss?.backAttacks ?? this.skill.backAttacks);
  baDmg = $derived(this.wss?.backAttackDamage ?? this.skill.backAttackDamage);
  fa = $derived(this.wss?.frontAttacks ?? this.skill.frontAttacks);
  faDmg = $derived(this.wss?.frontAttackDamage ?? this.skill.frontAttackDamage);
  casts = $derived(this.wss?.casts ?? this.skill.casts);
  maxDmg = $derived(this.wss?.maxDamage ?? this.skill.maxDamage);

  skillDps = $derived.by(() => {
    if (this.wss) return this.wss.dps;
    if (this.entity.encounter.live) {
      return Math.round(this.skill.totalDamage / (this.entity.encounter.duration / 1000));
    } else {
      return this.skill.dps;
    }
  });
  skillDpsString = $derived(abbreviateNumberSplit(this.skillDps));
  skillDamageString = $derived(abbreviateNumberSplit(this.totalDamage));
  skillUnbuffedDamage = $derived(this.totalDamage - sumRdpsReceived(this.skill, [1, 3, 5]));
  skillUnbuffedDps = $derived.by(() => {
    if (this.skillUnbuffedDamage === 0 || this.skillUnbuffedDamage === this.totalDamage) return this.skillDps;
    const dur = this.wss ? this.entity.wDurMs : this.entity.encounter.duration;
    return Math.round(this.skillUnbuffedDamage / (dur / 1000));
  });
  skillUnbuffedDamageString = $derived(abbreviateNumberSplit(this.skillUnbuffedDamage));
  skillUnbuffedDpsString = $derived(abbreviateNumberSplit(this.skillUnbuffedDps));

  skillBuffedDamage = $derived(sumRdpsContributed(this.skill, [1, 3, 5]));

  skillBuffedDps = $derived.by(() => {
    if (this.skillBuffedDamage === 0) return 0;
    const dur = this.wss ? this.entity.wDurMs : this.entity.encounter.duration;
    return Math.round(this.skillBuffedDamage / (dur / 1000));
  });

  skillDamageReduced = $derived(sumRdpsContributed(this.skill, [4, 6]));

  critPercentage = $derived.by(() => {
    if (this.hits > 0) {
      return customRound((this.crits / this.hits) * 100);
    }
    return "0";
  });
  critDmgPercentage = $derived.by(() => {
    if (this.hits > 0 && this.totalDamage > 0) {
      return customRound((this.critDmg / this.totalDamage) * 100);
    }
    return "0";
  });
  baPercentage = $derived.by(() => {
    if (this.hits > 0) {
      return customRound((this.ba / this.hits) * 100);
    }
    return "0";
  });
  badPercentage = $derived.by(() => {
    if (this.baDmg > 0) {
      return customRound((this.baDmg / this.totalDamage) * 100);
    }
    return "0";
  });
  faPercentage = $derived.by(() => {
    if (this.hits > 0) {
      return customRound((this.fa / this.hits) * 100);
    }
    return "0";
  });
  fadPercentage = $derived.by(() => {
    if (this.faDmg > 0) {
      return customRound((this.faDmg / this.totalDamage) * 100);
    }
    return "0";
  });
  averagePerCast = $derived(this.casts > 0 ? this.totalDamage / this.casts : 0);
  adjustedCrit = $derived.by(() => {
    if (settings.app.logs.breakdown.adjustedCritRate) {
      if (this.skill.adjustedCrit && !this.wss) {
        return customRound(this.skill.adjustedCrit * 100);
      } else {
        const filter = this.averagePerCast * 0.05;
        let adjustedCrits = 0;
        let adjustedHits = 0;
        if (this.skill.skillCastLog.length > 0) {
          const ws = this.entity.ws;
          const windowStartMs = ws ? this.entity.encounter.timeWindow?.startMs ?? 0 : 0;
          const windowEndMs = ws ? this.entity.encounter.timeWindow?.endMs ?? Infinity : Infinity;
          for (const c of this.skill.skillCastLog) {
            for (const h of c.hits) {
              if (ws && (h.timestamp < windowStartMs || h.timestamp >= windowEndMs)) continue;
              if (h.damage > filter) {
                adjustedCrits += h.crit ? 1 : 0;
                adjustedHits += 1;
              }
            }
          }
          if (adjustedHits > 0) {
            return customRound((adjustedCrits / adjustedHits) * 100);
          }
        }
      }
    }
    return undefined;
  });

  cooldownRatio = $derived.by(() => {
    if (this.skill.timeAvailable && this.skill.timeAvailable <= this.entity.encounter.duration) {
      return customRound((1 - this.skill.timeAvailable / this.entity.encounter.duration) * 100);
    }
  });

  constructor(skill: Skill, entity: EntityState) {
    this.entity = entity;
    this.skill = skill;
  }
}

export function sumRdpsReceived(skill: Skill, types: number[] = [1, 3, 5]): number {
  if (!skill.rdpsReceived) return 0;
  let sum = 0;
  for (const t of types) {
    const buffedByType = skill.rdpsReceived[t];
    if (!buffedByType) continue;
    for (const value of Object.values(buffedByType)) {
      sum += value;
    }
  }
  return sum;
}

// sums up all rdps contributed values for types 1, 3, and 5 (ap buff, brand, identity, t)
export function sumRdpsContributed(skill: Skill, types: number[] = [1, 3, 5]): number {
  if (!skill.rdpsContributed) return 0;

  let sum = 0;
  for (const t of types) {
    const value = skill.rdpsContributed[t];
    if (!value) continue;

    sum += value;
  }
  return sum;
}
