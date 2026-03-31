import { getSkillCastSupportBuffs, hyperAwakeningIds } from "$lib/utils/buffs";
import type { Encounter, Entity } from "$lib/types";

export interface WindowedEntityStats {
  // DamageStats overrides
  damageDealt: number;
  dps: number;
  unbuffedDamage: number;
  unbuffedDps: number;
  critDamage: number;
  backAttackDamage: number;
  frontAttackDamage: number;
  buffedBySupport: number;
  buffedByIdentity: number;
  debuffedBySupport: number;
  buffedByHat: number;
  hyperAwakeningDamage: number;
  specialDamage: number; // for damageDealtWithoutSpecial
  stagger: number;
  rdpsDamageReceived: number;
  rdpsDamageReceivedSupport: number;
  deaths: number;
  // SkillStats overrides
  hits: number;
  crits: number;
  backAttacks: number;
  frontAttacks: number;
  hitsSpecialOrHa: number; // for hitsWithoutSpecial denominator
}

export function computeWindowedEntityStats(
  entity: Entity,
  encounter: Encounter,
  windowStartMs: number, // relative to fightStart
  windowEndMs: number,
  windowDurationMs: number
): WindowedEntityStats {
  const result: WindowedEntityStats = {
    damageDealt: 0,
    dps: 0,
    unbuffedDamage: 0,
    unbuffedDps: 0,
    critDamage: 0,
    backAttackDamage: 0,
    frontAttackDamage: 0,
    buffedBySupport: 0,
    buffedByIdentity: 0,
    debuffedBySupport: 0,
    buffedByHat: 0,
    hyperAwakeningDamage: 0,
    specialDamage: 0,
    stagger: 0,
    rdpsDamageReceived: 0,
    rdpsDamageReceivedSupport: 0,
    deaths: 0,
    hits: 0,
    crits: 0,
    backAttacks: 0,
    frontAttacks: 0,
    hitsSpecialOrHa: 0
  };

  const fightStart = encounter.fightStart;
  // hit.timestamp is relative to fightStart (ms), so compare directly against windowStartMs/windowEndMs
  // deathInfo.deathTime is absolute, so keep fightStart-offset variants for that check
  const absoluteWindowStart = fightStart + windowStartMs;
  const absoluteWindowEnd = fightStart + windowEndMs;

  for (const skill of Object.values(entity.skills)) {
    const isSpecial = skill.special === true;
    const isHa = skill.isHyperAwakening || hyperAwakeningIds.has(skill.id);

    for (const cast of skill.skillCastLog) {
      for (const hit of cast.hits) {
        if (hit.timestamp < windowStartMs || hit.timestamp >= windowEndMs) continue;

        result.damageDealt += hit.damage;
        result.unbuffedDamage += hit.unbuffedDamage ?? hit.damage;
        result.rdpsDamageReceived += hit.rdpsDamageReceived;
        result.rdpsDamageReceivedSupport += hit.rdpsDamageReceivedSupport;
        result.hits++;

        if (hit.crit) {
          result.crits++;
          result.critDamage += hit.damage;
        }
        if (hit.backAttack) {
          result.backAttacks++;
          result.backAttackDamage += hit.damage;
        }
        if (hit.frontAttack) {
          result.frontAttacks++;
          result.frontAttackDamage += hit.damage;
        }
        if (hit.stagger) {
          result.stagger += hit.stagger;
        }
        if (isSpecial || isHa) {
          result.hitsSpecialOrHa++;
        }
        if (isSpecial) {
          result.specialDamage += hit.damage;
        }
        if (isHa) {
          result.hyperAwakeningDamage += hit.damage;
        }

        const supportBuffs = getSkillCastSupportBuffs(hit, encounter.encounterDamageStats);
        result.buffedBySupport += supportBuffs.buff;
        result.debuffedBySupport += supportBuffs.brand;
        result.buffedByIdentity += supportBuffs.identity;
        result.buffedByHat += supportBuffs.hat;
      }
    }
  }

  const durationSec = windowDurationMs / 1000;
  result.dps = durationSec > 0 ? Math.round(result.damageDealt / durationSec) : 0;
  result.unbuffedDps = durationSec > 0 ? Math.round(result.unbuffedDamage / durationSec) : 0;

  // Deaths: use deathInfo timestamps when available, fall back to full-encounter count
  if (entity.damageStats.deathInfo && entity.damageStats.deathInfo.length > 0) {
    result.deaths = entity.damageStats.deathInfo.filter(
      (d) => d.deathTime >= absoluteWindowStart && d.deathTime < absoluteWindowEnd
    ).length;
  } else {
    result.deaths = entity.damageStats.deaths;
  }

  return result;
}
