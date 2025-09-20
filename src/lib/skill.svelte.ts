import { abbreviateNumberSplit, customRound } from "$lib/utils";
import type { EntityState } from "./entity.svelte";
import { settings } from "./stores.svelte";
import type { Skill } from "./types";

export class SkillState {
  skill: Skill = $state()!;
  entity: EntityState = $state()!;

  skillDps = $derived.by(() => {
    if (this.entity.encounter.live) {
      return Math.round(this.skill.totalDamage / (this.entity.encounter.duration / 1000));
    } else {
      return this.skill.dps;
    }
  });
  skillDpsString = $derived(abbreviateNumberSplit(this.skillDps));
  skillDamageString = $derived(abbreviateNumberSplit(this.skill.totalDamage));
  skillUnbuffedDamage = $derived.by(() => {
    if (!this.skill.rdpsReceived) return 0;

    let sum = 0;
    for (const rdpsType of [1, 3, 5]) {
      const buffedBySkill = this.skill.rdpsReceived[rdpsType];
      if (!buffedBySkill) continue;
      for (const value of Object.values(buffedBySkill)) {
        sum += value;
      }
    }
    return this.skill.totalDamage - sum;
  });
  skillUnbuffedDps = $derived.by(() => {
    // the dps calculated here can slightly differ from one calculated in backend (prolly due to time/rounding? idk)
    // so returning pre-calculated dps if unbuffed damage equals total damage
    if (this.skillUnbuffedDamage === this.skill.totalDamage) return this.skillDps;
    return Math.round(this.skillUnbuffedDamage / (this.entity.encounter.duration / 1000));
  });
  skillUnbuffedDamageString = $derived(abbreviateNumberSplit(this.skillUnbuffedDamage));
  skillUnbuffedDpsString = $derived(abbreviateNumberSplit(this.skillUnbuffedDps));

  skillBuffedDamage = $derived(sumRdpsContributed(this.skill, [1, 3, 5]));

  skillBuffedDps = $derived.by(() => {
    if (this.skillBuffedDamage === 0) return 0;
    return Math.round(this.skillBuffedDamage / (this.entity.encounter.duration / 1000));
  });

  skillDamageReduced = $derived(sumRdpsContributed(this.skill, [4, 6]));

  critPercentage = $derived.by(() => {
    if (this.skill.hits > 0) {
      return customRound((this.skill.crits / this.skill.hits) * 100);
    }
    return "0";
  });
  critDmgPercentage = $derived.by(() => {
    if (this.skill.hits > 0 && this.skill.totalDamage > 0) {
      return customRound((this.skill.critDamage / this.skill.totalDamage) * 100);
    }
    return "0";
  });
  baPercentage = $derived.by(() => {
    if (this.skill.hits > 0) {
      return customRound((this.skill.backAttacks / this.skill.hits) * 100);
    }
    return "0";
  });
  badPercentage = $derived.by(() => {
    if (this.skill.backAttackDamage > 0) {
      return customRound((this.skill.backAttackDamage / this.skill.totalDamage) * 100);
    }
    return "0";
  });
  faPercentage = $derived.by(() => {
    if (this.skill.hits > 0) {
      return customRound((this.skill.frontAttacks / this.skill.hits) * 100);
    }
    return "0";
  });
  fadPercentage = $derived.by(() => {
    if (this.skill.frontAttackDamage > 0) {
      return customRound((this.skill.frontAttackDamage / this.skill.totalDamage) * 100);
    }
    return "0";
  });
  averagePerCast = $derived(this.skill.totalDamage / this.skill.casts);
  adjustedCrit = $derived.by(() => {
    if (settings.app.logs.breakdown.adjustedCritRate) {
      if (this.skill.adjustedCrit) {
        return customRound(this.skill.adjustedCrit * 100);
      } else {
        const filter = this.averagePerCast * 0.05;
        let adjustedCrits = 0;
        let adjustedHits = 0;
        if (this.skill.skillCastLog.length > 0) {
          for (const c of this.skill.skillCastLog) {
            for (const h of c.hits) {
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
