import { getSkillCastSupportBuffs, hyperAwakeningIds } from "$lib/utils/buffs";
import type { Encounter, Entity } from "$lib/types";

export interface WindowedSkillStats {
  totalDamage: number;
  dps: number;
  hits: number;
  casts: number;
  crits: number;
  critDamage: number;
  backAttacks: number;
  backAttackDamage: number;
  frontAttacks: number;
  frontAttackDamage: number;
  buffedBy: { [key: number]: number };
  debuffedBy: { [key: number]: number };
  maxDamage: number;
  stagger: number;
}

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
  // per-buff damage (mirrors entity.damageStats.buffedBy/debuffedBy)
  buffedBy: { [key: number]: number };
  debuffedBy: { [key: number]: number };
  // per-skill stats
  skillStats: Map<number, WindowedSkillStats>;
}

function newSkillStats(): WindowedSkillStats {
  return {
    totalDamage: 0,
    dps: 0,
    hits: 0,
    casts: 0,
    crits: 0,
    critDamage: 0,
    backAttacks: 0,
    backAttackDamage: 0,
    frontAttacks: 0,
    frontAttackDamage: 0,
    buffedBy: {},
    debuffedBy: {},
    maxDamage: 0,
    stagger: 0
  };
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
    hitsSpecialOrHa: 0,
    buffedBy: {},
    debuffedBy: {},
    skillStats: new Map()
  };

  const fightStart = encounter.fightStart;
  // hit.timestamp is relative to fightStart (ms), so compare directly against windowStartMs/windowEndMs
  // deathInfo.deathTime is absolute, so keep fightStart-offset variants for that check
  const absoluteWindowStart = fightStart + windowStartMs;
  const absoluteWindowEnd = fightStart + windowEndMs;

  for (const skill of Object.values(entity.skills)) {
    const isSpecial = skill.special === true;
    const isHa = skill.isHyperAwakening || hyperAwakeningIds.has(skill.id);

    let ss: WindowedSkillStats | undefined;

    for (const cast of skill.skillCastLog) {
      let castHadHitInWindow = false;

      for (const hit of cast.hits) {
        if (hit.timestamp < windowStartMs || hit.timestamp >= windowEndMs) continue;

        if (!ss) {
          ss = newSkillStats();
          result.skillStats.set(skill.id, ss);
        }

        castHadHitInWindow = true;

        result.damageDealt += hit.damage;
        result.unbuffedDamage += hit.unbuffedDamage ?? hit.damage;
        result.rdpsDamageReceived += hit.rdpsDamageReceived;
        result.rdpsDamageReceivedSupport += hit.rdpsDamageReceivedSupport;
        result.hits++;

        // per-skill stats
        ss.totalDamage += hit.damage;
        ss.hits++;
        if (hit.damage > ss.maxDamage) ss.maxDamage = hit.damage;
        ss.stagger += hit.stagger ?? 0;

        if (hit.crit) {
          result.crits++;
          result.critDamage += hit.damage;
          ss.crits++;
          ss.critDamage += hit.damage;
        }
        if (hit.backAttack) {
          result.backAttacks++;
          result.backAttackDamage += hit.damage;
          ss.backAttacks++;
          ss.backAttackDamage += hit.damage;
        }
        if (hit.frontAttack) {
          result.frontAttacks++;
          result.frontAttackDamage += hit.damage;
          ss.frontAttacks++;
          ss.frontAttackDamage += hit.damage;
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

        // per-buff damage for buff percentage calculations
        for (const buffId of hit.buffedBy) {
          result.buffedBy[buffId] = (result.buffedBy[buffId] ?? 0) + hit.damage;
          ss.buffedBy[buffId] = (ss.buffedBy[buffId] ?? 0) + hit.damage;
        }
        for (const buffId of hit.debuffedBy) {
          result.debuffedBy[buffId] = (result.debuffedBy[buffId] ?? 0) + hit.damage;
          ss.debuffedBy[buffId] = (ss.debuffedBy[buffId] ?? 0) + hit.damage;
        }
      }

      if (castHadHitInWindow && ss) {
        ss.casts++;
      }
    }

    // compute per-skill dps
    if (ss) {
      const durationSec = windowDurationMs / 1000;
      ss.dps = durationSec > 0 ? Math.round(ss.totalDamage / durationSec) : 0;
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
