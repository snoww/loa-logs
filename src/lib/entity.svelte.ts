import {
  abbreviateNumberSplit,
  customRound,
  formatPlayerName,
  getBaseDamage,
  getEstherFromNpcId,
  getRDamage
} from "$lib/utils";

export type SkillSort = "damage" | "buffed" | "stagger";
import { cardIds } from "./constants/cards";
import { classNameToClassId } from "./constants/classes";
import type { EncounterState } from "./encounter.svelte";
import { sumUdpsContributed } from "./skill.svelte";
import { settings } from "./stores.svelte";
import { type Entity, EntityType, type IncapacitatedEvent, type Skill } from "./types";
import { hyperAwakeningIds, supportSkills } from "./utils/buffs";

export const IDENTITY_BRAND_SKILL_ID = -210230;

function hasLegacyUdpsContribution(entity: Entity): boolean {
  return Object.values(entity.skills).some((skill) => sumUdpsContributed(skill, [1, 3, 5]) > 0);
}

function getContributionScopeMembers(encounter: EncounterState, playerName: string): Entity[] {
  const parties = encounter.parties;
  if (parties.length > 0) {
    return parties.find((party) => party.some((p) => p.name === playerName)) ?? encounter.playersOnly;
  }
  return encounter.playersOnly;
}

function getContributionScopeDpsPlayers(encounter: EncounterState, playerName: string): Entity[] {
  return getContributionScopeMembers(encounter, playerName).filter((player) => !hasLegacyUdpsContribution(player));
}

export class EntityState {
  entity: Entity = $state()!;
  encounter: EncounterState = $state()!;

  name: string = $derived.by(() => {
    if (!this.entity) return "";
    if (this.entity.entityType === EntityType.ESTHER) {
      return getEstherFromNpcId(this.entity.npcId);
    } else {
      return formatPlayerName(this.entity);
    }
  });

  color = $derived.by(() => {
    if (this.entity.entityType === EntityType.ESTHER) {
      return "#4dc8d0";
    }
    if (Object.hasOwn(settings.classColors, this.entity.class)) {
      if (settings.app.general.constantLocalPlayerColor && this.encounter.localPlayer == this.entity.name) {
        return settings.classColors["Local"];
      } else {
        return settings.classColors[this.entity.class];
      }
    }

    return "#fff";
  });

  dps = $derived.by(() => {
    if (this.encounter.live) {
      return Math.round(this.entity.damageStats.damageDealt / (this.encounter.duration / 1000));
    } else {
      return this.entity.damageStats.dps;
    }
  });

  dpsString = $derived.by(() => {
    if (this.encounter.duration > 0 || !this.encounter.live) {
      return abbreviateNumberSplit(this.dps);
    } else {
      return abbreviateNumberSplit(0);
    }
  });

  damageDealt = $derived(this.entity.damageStats.damageDealt);
  damageDealtString = $derived(abbreviateNumberSplit(this.damageDealt));
  baseDamage = $derived(getBaseDamage(this.entity.damageStats));
  rdamage = $derived(getRDamage(this.entity.damageStats));
  ndps = $derived.by(() => {
    if (this.encounter.duration <= 0) return 0;
    return Math.round(this.baseDamage / (this.encounter.duration / 1000));
  });
  ndpsString = $derived(abbreviateNumberSplit(this.ndps));
  rdps = $derived.by(() => {
    if (this.encounter.duration <= 0) return 0;
    return Math.round(this.rdamage / (this.encounter.duration / 1000));
  });
  rdpsString = $derived(abbreviateNumberSplit(this.rdps));
  damageDealtWithoutSpecial = $derived(
    this.damageDealt -
      Object.values(this.entity.skills)
        .filter((skill) => skill.special)
        .reduce((acc, skill) => acc + skill.totalDamage, 0)
  );
  damageDealtWithoutSpecialOrHa = $derived(
    this.damageDealtWithoutSpecial - (this.entity.damageStats.hyperAwakeningDamage ?? 0)
  );
  damagePercentage = $derived(((this.damageDealt / this.encounter.totalDamageDealt) * 100).toFixed(1));
  damageTakenString = $derived(abbreviateNumberSplit(this.entity.damageStats.damageTaken));
  unbuffedDps = $derived.by(() => {
    if (this.encounter.live) {
      if (this.entity.damageStats.unbuffedDamage === 0) return this.dps;
      return Math.round(this.entity.damageStats.unbuffedDamage / (this.encounter.duration / 1000));
    } else {
      return this.entity.damageStats.unbuffedDps ? this.entity.damageStats.unbuffedDps : this.dps;
    }
  });

  hitsWithoutSpecial = $derived(
    this.entity.skillStats.hits -
      Object.values(this.entity.skills)
        .filter((skill) => skill.special || skill.isHyperAwakening || hyperAwakeningIds.has(skill.id))
        .reduce((acc, skill) => acc + skill.hits, 0)
  );

  deadFor: string = $derived.by(() => {
    if (this.entity.isDead) {
      return Math.abs((this.encounter.end - this.entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
    }
    return "";
  });

  critPercentage = $derived.by(() => {
    if (this.hitsWithoutSpecial > 0) {
      return customRound((this.entity.skillStats.crits / this.hitsWithoutSpecial) * 100);
    }
    return "0";
  });
  critDmgPercentage = $derived.by(() => {
    if (this.hitsWithoutSpecial > 0) {
      return customRound((this.entity.damageStats.critDamage / this.damageDealtWithoutSpecialOrHa) * 100);
    }
    return "0";
  });
  baPercentage = $derived.by(() => {
    if (this.hitsWithoutSpecial > 0) {
      return customRound((this.entity.skillStats.backAttacks / this.hitsWithoutSpecial) * 100);
    }

    return "0";
  });
  badPercentage = $derived.by(() => {
    if (this.entity.damageStats.backAttackDamage > 0) {
      return customRound((this.entity.damageStats.backAttackDamage / this.damageDealtWithoutSpecialOrHa) * 100);
    }
    return "0";
  });
  faPercentage = $derived.by(() => {
    if (this.hitsWithoutSpecial > 0) {
      return customRound((this.entity.skillStats.frontAttacks / this.hitsWithoutSpecial) * 100);
    }
    return "0";
  });
  fadPercentage = $derived.by(() => {
    if (this.entity.damageStats.frontAttackDamage > 0) {
      return customRound((this.entity.damageStats.frontAttackDamage / this.damageDealtWithoutSpecialOrHa) * 100);
    }
    return "0";
  });

  incapacitatedTimeMs = $derived.by(() => {
    const events = this.entity.damageStats.incapacitations;
    return {
      total: this.computeIncapacitatedTime(events),
      knockDown: this.computeIncapacitatedTime(events.filter((event) => event.type === "FALL_DOWN")),
      cc: this.computeIncapacitatedTime(events.filter((event) => event.type === "CROWD_CONTROL"))
    };
  });

  skillSort: SkillSort = $state("buffed");

  /**
   * Computes the "identity brand" bDMG — brand damage misattributed to identity skills
   * because the support's identity (Moonfall, Serenade, Release Light, etc.) can apply
   * brand on the boss entity. The game reports this bonus damage under the identity skill's
   * udpsContributed instead of brand, inflating identity and deflating brand.
   *
   * Returns null if the support has no identity-applied brand this encounter.
   */
  identityBrandInfo = $derived.by(() => {
    if (!this.encounter.encounter) return null;
    const rawSkills = Object.values(this.entity.skills);
    const isSupport = rawSkills.some((s) => sumUdpsContributed(s, [1, 3, 5]) > 0);
    if (!isSupport) return null;

    const enc = this.encounter.encounter;
    const allDebuffs = enc.encounterDamageStats.debuffs;

    // Step 1: Partition brand debuffs into "regular brand" vs "identity brand".
    // All brand debuffs share uniqueGroup 210230. Identity brand = sourceSkill is one of
    // the known identity skills (Moonfall, Serenade of Courage, Blessed Aura, Release Light).
    // Everything else is regular brand (Sonatina, Drawing Orchids, Dissonance, etc.).
    const identityBrandDebuffIds = new Set<number>();
    const regularBrandDebuffIds = new Set<number>();
    const entityClassId = classNameToClassId[this.entity.class];
    for (const [idStr, debuff] of Object.entries(allDebuffs)) {
      if (debuff.uniqueGroup === 210230 && debuff.source.skill?.classId === entityClassId) {
        const srcId = debuff.source.skill?.id ?? 0;
        if (supportSkills.identityBrandSources.includes(srcId)) {
          identityBrandDebuffIds.add(Number(idStr));
        } else {
          regularBrandDebuffIds.add(Number(idStr));
        }
      }
    }

    if (identityBrandDebuffIds.size === 0) return null;

    // Step 2: Sum window-damage across DPS party members for each brand type.
    const name = this.entity.name;
    const parties = this.encounter.parties;
    const partyMembers =
      parties.length > 0
        ? (parties.find((party) => party.some((p) => p.name === name)) ?? this.encounter.playersOnly)
        : this.encounter.playersOnly;

    let identityBrandWindowDmg = 0;
    let regularBrandWindowDmg = 0;
    for (const player of partyMembers) {
      const playerIsSupport = Object.values(player.skills).some((s) => sumUdpsContributed(s, [1, 3, 5]) > 0);
      if (playerIsSupport) continue;
      for (const [idStr, dmg] of Object.entries(player.damageStats.debuffedBy)) {
        const id = Number(idStr);
        if (identityBrandDebuffIds.has(id)) identityBrandWindowDmg += dmg;
        else if (regularBrandDebuffIds.has(id)) regularBrandWindowDmg += dmg;
      }
    }

    if (identityBrandWindowDmg === 0 || regularBrandWindowDmg === 0) return null;

    // Step 3: Total regular brand bDMG from this support's skills (type 3 = boss debuff).
    const regularBrandBDmg = rawSkills.reduce((acc, s) => acc + (s.rdpsContributed[3] ?? 0), 0);
    if (regularBrandBDmg === 0) return null;

    // Step 4: Extrapolate: how much brand bDMG was hidden in identity udpsContributed?
    // identityBrandBDmg = regularBrandBDmg * (identityBrandWindowDmg / regularBrandWindowDmg)
    const identityBrandBDmg = Math.round(regularBrandBDmg * (identityBrandWindowDmg / regularBrandWindowDmg));
    if (identityBrandBDmg <= 0) return null;

    // Step 5: Identify which of this support's skills are identity skills by matching
    // their skill id against the known identity brand source list.
    const identitySkillIds = new Set<number>(supportSkills.identityBrandSources);

    const totalIdentityUdps = rawSkills.reduce(
      (acc, s) => acc + (identitySkillIds.has(s.id) ? (s.rdpsContributed[1] ?? 0) : 0),
      0
    );
    if (totalIdentityUdps === 0) return null;

    const identityCasts = rawSkills.reduce((acc, s) => acc + (identitySkillIds.has(s.id) ? s.casts : 0), 0);

    return { bDmg: identityBrandBDmg, totalIdentityUdps, identitySkillIds, casts: identityCasts };
  });

  skills = $derived.by(() => {
    if (!this.entity) return [];
    const skillValues = Object.values(this.entity.skills);
    if (this.entity.class === "Arcanist") {
      const sortFn =
        this.skillSort === "stagger"
          ? (a: Skill, b: Skill) => b.stagger - a.stagger
          : (a: Skill, b: Skill) => b.totalDamage - a.totalDamage;
      return skillValues.sort(sortFn).filter((skill) => !cardIds.includes(skill.id));
    }

    // For supports with identity brand, inject adjusted identity skills + synthetic skill.
    const info = this.identityBrandInfo;
    let adjustedSkills: Skill[];
    if (info) {
      adjustedSkills = skillValues.map((skill) => {
        if (!info.identitySkillIds.has(skill.id)) return skill;
        // Subtract the identity brand bDMG proportionally from each identity skill's type-1 rdps.
        const skillIdentityUdps = skill.rdpsContributed[1] ?? 0;
        if (skillIdentityUdps <= 0) return skill;
        const reduction = Math.round(info.bDmg * (skillIdentityUdps / info.totalIdentityUdps));
        return {
          ...skill,
          rdpsContributed: {
            ...skill.rdpsContributed,
            1: Math.max(0, skillIdentityUdps - reduction)
          }
        };
      });
      // Synthetic "identity brand" skill — bDMG stored as type 3 (boss debuff, same as brand).
      const identityBrandMeta: Record<string, { name: string; icon: string }> = {
        Artist: { name: "Brand Enhancement (Moonfall Brand)", icon: "ark_passive_yy_6.png" },
        Paladin: { name: "Light's Vestige (Blessed Aura Brand)", icon: "ark_passive_hk_7.png" },
        Bard: { name: "Serenade of Branding (Serenade Brand)", icon: "ark_passive_bd_9.png" },
        Valkyrie: { name: "Liberator's Sign (Release Light Brand)", icon: "ark_passive_hkf_10.png" }
      };
      const meta = identityBrandMeta[this.entity.class] ?? { name: "Identity Brand", icon: "" };
      const syntheticSkill: Skill = {
        id: IDENTITY_BRAND_SKILL_ID,
        name: meta.name,
        icon: meta.icon,
        totalDamage: 0,
        maxDamage: 0,
        maxDamageCast: 0,
        buffedBy: {},
        debuffedBy: {},
        buffedBySupport: 0,
        buffedByIdentity: 0,
        buffedByHat: 0,
        debuffedBySupport: 0,
        casts: info.casts,
        hits: 0,
        crits: 0,
        critDamage: 0,
        backAttacks: 0,
        frontAttacks: 0,
        backAttackDamage: 0,
        frontAttackDamage: 0,
        dps: 0,
        castLog: [],
        skillCastLog: [],
        stagger: 0,
        rdpsReceived: {},
        rdpsContributed: { 3: info.bDmg },
        rdpsDamageReceived: 0
      };
      adjustedSkills.push(syntheticSkill);
    } else {
      adjustedSkills = skillValues;
    }

    const isSupport = adjustedSkills.some((skill) => sumUdpsContributed(skill, [1, 3, 5]) > 0);
    if (this.skillSort === "stagger") return adjustedSkills.sort((a, b) => b.stagger - a.stagger);
    if (this.skillSort === "buffed" && isSupport && this.encounter.curSettings.breakdown.unbuffedDamage)
      return adjustedSkills.sort((a, b) => sumUdpsContributed(b, [1, 3, 5]) - sumUdpsContributed(a, [1, 3, 5]));
    return adjustedSkills.sort((a, b) => b.totalDamage - a.totalDamage);
  });

  isSupport = $derived(this.skills.some((skill) => sumUdpsContributed(skill, [1, 3, 5]) > 0));

  private skillSortValue(skill: Skill): number {
    if (this.skillSort === "stagger") return skill.stagger ?? 0;
    if (this.skillSort === "buffed" && this.isSupport && this.encounter.curSettings.breakdown.unbuffedDamage)
      return sumUdpsContributed(skill, [1, 3, 5]);
    return skill.totalDamage;
  }

  mostDamageSkill = $derived(this.skills.length > 0 ? this.skillSortValue(this.skills[0]!) : 0);

  skillDamagePercentages = $derived(
    this.skills.map((skill) =>
      this.mostDamageSkill > 0 ? (this.skillSortValue(skill) / this.mostDamageSkill) * 100 : 0
    )
  );
  anyBackAttacks = $derived(this.skills.some((skill) => skill.backAttacks > 0));
  anyFrontAttacks = $derived(this.skills.some((skill) => skill.frontAttacks > 0));
  anySupportBuff = $derived(this.skills.some((skill) => skill.buffedBySupport > 0));
  anySupportIdentity = $derived(this.skills.some((skill) => skill.buffedByIdentity > 0));
  anySupportBrand = $derived(this.skills.some((skill) => skill.debuffedBySupport > 0));
  anySupportHat = $derived(this.skills.some((skill) => skill.buffedByHat > 0));

  anyCooldownRatio = $derived(this.skills.some((skill) => skill.timeAvailable));
  anyStagger = $derived(this.entity.damageStats.stagger > 0);
  anyUnbuffedDamage = $derived(
    this.entity.damageStats.unbuffedDamage > 0 &&
      this.entity.damageStats.unbuffedDamage !== this.entity.damageStats.damageDealt
  );

  hasUdpsContributions = $derived(
    Object.values(this.entity.skills).some((skill) => sumUdpsContributed(skill, [1, 3, 5]) > 0)
  );

  /**
   * Weighted average buff contribution % for this support, scoped to their own party.
   * Falls back to all players if party info is unavailable.
   * Equivalent to sum(buffed) / sum(damage) across DPS party members with unbuffed data.
   */
  supportContribPercent = $derived.by(() => {
    const partyDpsPlayers = getContributionScopeDpsPlayers(this.encounter, this.entity.name).filter(
      (player) =>
        player.damageStats.unbuffedDamage > 0 && player.damageStats.unbuffedDamage !== player.damageStats.damageDealt
    );
    const partyTotalDmg = partyDpsPlayers.reduce((acc, p) => acc + p.damageStats.damageDealt, 0);
    const partyTotalUnbuffed = partyDpsPlayers.reduce((acc, p) => acc + p.damageStats.unbuffedDamage, 0);
    if (partyTotalDmg === 0) return 0;
    return ((partyTotalDmg - partyTotalUnbuffed) / partyTotalDmg) * 100;
  });
  rdpsContribPercent = $derived.by(() => {
    if (this.isSupport) {
      const partyTotalDmg = getContributionScopeDpsPlayers(this.encounter, this.entity.name).reduce(
        (acc, player) => acc + player.damageStats.damageDealt,
        0
      );
      if (partyTotalDmg === 0) return 0;
      return (this.entity.damageStats.rdpsDamageGiven / partyTotalDmg) * 100;
    }
    if (this.damageDealt === 0) return 0;
    return (this.entity.damageStats.rdpsDamageReceived / this.damageDealt) * 100;
  });
  rdpsContribDamage = $derived.by(() =>
    this.isSupport ? this.entity.damageStats.rdpsDamageGiven : this.entity.damageStats.rdpsDamageReceived
  );
  hasDrContributions = $derived(
    Object.values(this.entity.skills).some((skill) => sumUdpsContributed(skill, [4, 6]) > 0)
  );
  totalDamageBuffed = $derived.by(() => {
    if (!this.hasUdpsContributions) return 0;
    return this.skills.reduce((acc, skill) => acc + sumUdpsContributed(skill), 0);
  });
  totalDamageBuffedString = $derived(abbreviateNumberSplit(this.totalDamageBuffed));
  totalDpsBuffed = $derived.by(() => {
    if (this.totalDamageBuffed <= 0) return 0;
    return Math.round(this.totalDamageBuffed / (this.encounter.duration / 1000));
  });
  totalDpsBuffedString = $derived(abbreviateNumberSplit(this.totalDpsBuffed));
  totalDamageReduced = $derived.by(() => {
    if (!this.hasDrContributions) return 0;
    return this.skills.reduce((acc, skill) => acc + sumUdpsContributed(skill, [4, 6]), 0);
  });
  totalDamageReducedString = $derived(abbreviateNumberSplit(this.totalDamageReduced));

  constructor(entity: Entity, enc: EncounterState) {
    this.entity = entity;
    this.encounter = enc;
  }

  // compute total sum of time spent incapacitated for given events, accounting for overlap
  computeIncapacitatedTime(events: IncapacitatedEvent[]) {
    if (!events.length) return 0;

    const enc = this.encounter;
    let totalTimeIncapacitated = 0;

    function addInterval(ivStart: number, ivEnd: number) {
      // clamp interval to the most recent damage event, such that
      // we don't count time spent incapacitated that has yet to happen
      ivStart = Math.min(ivStart, enc.end);
      ivEnd = Math.min(ivEnd, enc.end);
      totalTimeIncapacitated += Math.max(0, ivEnd - ivStart);
    }

    // collapse concurrent events so that we don't count the same time twice
    // note that the events array is guaranteed to be sorted by start time
    let curIntervalStart = events[0].timestamp;
    let curIntervalEnd = events[0].timestamp + events[0].duration;
    for (let i = 1; i < events.length; i++) {
      const event = events[i];

      // if this event starts after the current interval ends, add the current interval to the total
      if (event.timestamp > curIntervalEnd) {
        addInterval(curIntervalStart, curIntervalEnd);
        curIntervalStart = event.timestamp;
        curIntervalEnd = event.timestamp + event.duration;
      } else {
        // otherwise, extend the current interval
        curIntervalEnd = Math.max(curIntervalEnd, event.timestamp + event.duration);
      }
    }

    // add the last interval to the total
    addInterval(curIntervalStart, curIntervalEnd);
    return totalTimeIncapacitated;
  }
}
