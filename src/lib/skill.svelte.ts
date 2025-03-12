import type { Skill } from "./types";
import type { EntityState } from "./entity.svelte";
import { abbreviateNumberSplit, round } from "./utils/numbers";

export class SkillState {
    skill: Skill = $state()!;
    entity: EntityState = $state()!;

    skillDps = $derived.by(() => {
        if (this.entity.enc.live) {
            return Math.round(this.skill.totalDamage / (this.entity.enc.duration / 1000));
        } else {
            return this.skill.dps;
        }
    });
    skillDpsString = $derived(abbreviateNumberSplit(this.skillDps));
    skillDamageString = $derived(abbreviateNumberSplit(this.skill.totalDamage));
    critPercentage = $derived.by(() => {
        if (this.skill.hits > 0) {
            return round((this.skill.crits / this.skill.hits) * 100);
        }
        return "0.0";
    });
    critDmgPercentage = $derived.by(() => {
        if (this.skill.hits > 0) {
            return round((this.skill.critDamage / this.skill.totalDamage) * 100);
        }
        return "0.0";
    });
    baPercentage = $derived.by(() => {
        if (this.entity.enc.curSettings.positionalDmgPercent && this.skill.backAttackDamage > 0) {
            return round((this.skill.backAttackDamage / this.skill.totalDamage) * 100);
        }
        return round((this.skill.backAttacks / this.skill.hits) * 100);
    });
    faPercentage = $derived.by(() => {
        if (this.entity.enc.curSettings.positionalDmgPercent && this.skill.frontAttackDamage > 0) {
            return round((this.skill.frontAttackDamage / this.skill.totalDamage) * 100);
        }
        return round((this.skill.frontAttacks / this.skill.hits) * 100);
    });
    averagePerCast = $derived(this.skill.totalDamage / this.skill.casts);
    adjustedCrit = $derived.by(() => {
        if (this.entity.enc.curSettings.breakdown.adjustedCritRate) {
            if (this.skill.adjustedCrit) {
                return round(this.skill.adjustedCrit * 100);
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
                        return round((adjustedCrits / adjustedHits) * 100);
                    }
                }
            }
        }
        return undefined;
    });

    constructor(skill: Skill, entity: EntityState) {
        this.entity = entity;
        this.skill = skill;
    }
}
