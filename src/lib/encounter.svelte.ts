import { type ContributionSplit, type Encounter, type Entity, EntityType } from "$lib/types";
import { classNameToClassId } from "./constants/classes";
import { sumUdpsContributed } from "./skill.svelte";
import { settings } from "./stores.svelte";
import { getBaseDamage, getRDamage, timestampToMinutesAndSeconds } from "./utils";
import { supportSkills } from "./utils/buffs";

export interface IdentityBrandInfo {
  bDmg: number;
  totalIdentityUdps: number;
  identitySkillIds: Set<number>;
  casts: number;
}

export type PlayerSort = "dps" | "ndps" | "rdps" | "stagger";

export class EncounterState {
  live = false;

  encounter: Encounter | undefined = $state();
  curSettings = $derived(this.live ? settings.app.meter : settings.app.logs);
  playerSort: PlayerSort = $state("dps");
  end = $derived(this.encounter?.lastCombatPacket ?? 0);

  duration = $state(this.encounter?.duration ?? 0);
  region = $derived.by(() => {
    const region = this.encounter?.encounterDamageStats.misc?.region ?? this.encounter?.region ?? "";
    if (region && region === "EUC") {
      return "CE";
    }
    return region;
  });

  /**
   * Array of players in the encounter, irrespective of party membership, sorted by damage dealt (descending).
   *
   * If showEsther is enabled, the array will include the esthers in the encounter.
   */
  players = $derived.by(() => {
    if (!this.encounter) return [];
    const entities = Object.values(this.encounter.entities).filter((e) => {
      if (e.damageStats.damageDealt <= 0) return false;
      const isValidPlayer = e.entityType === EntityType.PLAYER && e.classId !== 0;
      if (settings.app.general.showEsther) {
        return isValidPlayer || e.entityType === EntityType.ESTHER;
      }
      return isValidPlayer;
    });
    switch (this.playerSort) {
      case "ndps":
        return entities.sort((a, b) => getBaseDamage(b.damageStats) - getBaseDamage(a.damageStats));
      case "rdps":
        return entities.sort((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
      case "stagger":
        return entities.sort((a, b) => b.damageStats.stagger - a.damageStats.stagger);
      default:
        return entities.sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
    }
  });

  /**
   * Array of players in the encounter, irrespective of party membership, sorted by damage dealt (descending).
   */
  playersOnly = $derived.by(() => {
    if (!this.encounter) return [];
    return Object.values(this.encounter.entities)
      .filter((e) => {
        if (e.damageStats.damageDealt <= 0) return false;

        return e.entityType === EntityType.PLAYER && e.classId !== 0;
      })
      .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
  });

  /**
   * Synthetic Dark Grenade entity, if any rDPS was attributed to it.
   * Has no self damage, only rdpsDamageGiven from dark grenade buff attribution
   */
  darkGrenade = $derived.by(() => {
    if (!this.encounter) return undefined;
    return Object.values(this.encounter.entities).find(
      (e) => e.entityType === EntityType.DARK_GRENADE && e.damageStats.rdpsDamageGiven > 0
    );
  });

  /**
   * Array of bosses in the encounter, sorted by damage dealt (descending).
   */
  bosses = $derived.by(() => {
    if (!this.encounter) return [];
    return Object.values(this.encounter.entities)
      .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.BOSS)
      .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
  });

  /**
   * The local player in the encounter.
   *
   * This is also the uploader of the log.
   */
  localPlayer = $derived(this.encounter?.localPlayer ?? "");

  // Single-pass aggregation over players. Each flag previously had its own
  // `.some()` derived, which meant ~14 passes over the players array on every
  // encounter update. Fold them into one pass and have the public derivations
  // read from this cache.
  aggregates = $derived.by(() => {
    let anyDead = false;
    let multipleDeaths = false;
    let anyFrontAtk = false;
    let anyBackAtk = false;
    let anyCounters = false;
    let anySupportBuff = false;
    let anySupportIdentity = false;
    let anySupportBrand = false;
    let anySupportHat = false;
    let anyPlayerIncapacitated = false;
    let anyStagger = false;
    let anyUnbuffedDamage = false;
    let anyRdpsContributionsRaw = false;
    const supportNames = new Set<string>();

    for (const p of this.players) {
      const ds = p.damageStats;
      const ss = p.skillStats;
      if (p.isDead) anyDead = true;
      if (ds.deaths > 0 && !p.isDead) multipleDeaths = true;
      if (ss.frontAttacks > 0) anyFrontAtk = true;
      if (ss.backAttacks > 0) anyBackAtk = true;
      if (ss.counters > 0) anyCounters = true;
      if (ds.buffedBySupport > 0) anySupportBuff = true;
      if (ds.buffedByIdentity > 0) anySupportIdentity = true;
      if (ds.debuffedBySupport > 0) anySupportBrand = true;
      if (ds.buffedByHat && ds.buffedByHat > 0) anySupportHat = true;
      if (ds.incapacitations && ds.incapacitations.length > 0) anyPlayerIncapacitated = true;
      if (ds.stagger > 0) anyStagger = true;
      if (ds.unbuffedDamage > 0) anyUnbuffedDamage = true;
      if (ds.rdpsDamageGiven > 0 || ds.rdpsDamageReceived > 0) anyRdpsContributionsRaw = true;

      for (const skill of Object.values(p.skills)) {
        if (sumUdpsContributed(skill, [1, 3, 5]) > 0) {
          supportNames.add(p.name);
          break;
        }
      }
    }

    return {
      anyDead,
      multipleDeaths,
      anyFrontAtk,
      anyBackAtk,
      anyCounters,
      anySupportBuff,
      anySupportIdentity,
      anySupportBrand,
      anySupportHat,
      anyPlayerIncapacitated,
      anyStagger,
      anyUnbuffedDamage,
      anyRdpsContributionsRaw,
      supportNames
    };
  });

  // this section is used for conditional headers in the damage table
  isSolo = $derived(this.players.length === 1);
  anyDead = $derived(this.aggregates.anyDead);
  multipleDeaths = $derived(this.aggregates.multipleDeaths);
  anyFrontAtk = $derived(this.aggregates.anyFrontAtk);
  anyBackAtk = $derived(this.aggregates.anyBackAtk);
  anyCounters = $derived(this.aggregates.anyCounters);
  anySupportBuff = $derived(this.aggregates.anySupportBuff);
  anySupportIdentity = $derived(this.aggregates.anySupportIdentity);
  anySupportBrand = $derived(this.aggregates.anySupportBrand);
  anySupportHat = $derived(this.aggregates.anySupportHat);
  anyPlayerIncapacitated = $derived(this.aggregates.anyPlayerIncapacitated);
  anyStagger = $derived(this.aggregates.anyStagger);
  anyUnbuffedDamage = $derived(this.aggregates.anyUnbuffedDamage);
  anyUdpsContributions = $derived(this.aggregates.supportNames.size > 0);
  supportNames = $derived(this.aggregates.supportNames);
  anyRdpsContributions = $derived(
    this.encounter?.encounterDamageStats.misc?.rdpsValid !== false && this.aggregates.anyRdpsContributionsRaw
  );

  /**
   * Computes per-support "identity brand" bDMG — brand damage misattributed to identity skills
   * because the support's identity (Moonfall, Serenade, Release Light, etc.) can apply
   * brand on the boss entity. The game reports this bonus damage under the identity skill's
   * udpsContributed instead of brand, inflating identity and deflating brand.
   *
   * Runs once per encounter update for every support in the encounter; `EntityState` just
   * looks up the result by player name. Supports without any identity-applied brand are
   * absent from the map.
   */
  identityBrandContextByPlayer = $derived.by(() => {
    const result = new Map<string, IdentityBrandInfo>();
    if (!this.encounter) return result;
    const supportNames = this.aggregates.supportNames;
    if (supportNames.size === 0) return result;

    const allDebuffs = this.encounter.encounterDamageStats.debuffs;
    const identityBrandSourceSet = new Set(supportSkills.identityBrandSources);
    const partiesList = this.parties;

    for (const name of supportNames) {
      const entity = this.encounter.entities[name];
      if (!entity) continue;

      // Step 1: Partition brand debuffs into "regular brand" vs "identity brand".
      // All brand debuffs share uniqueGroup 210230. Identity brand = sourceSkill is one of
      // the known identity skills (Moonfall, Serenade of Courage, Blessed Aura, Release Light).
      // Everything else is regular brand (Sonatina, Drawing Orchids, Dissonance, etc.).
      const entityClassId = classNameToClassId[entity.class];
      const identityBrandDebuffIds = new Set<number>();
      const regularBrandDebuffIds = new Set<number>();
      for (const [idStr, debuff] of Object.entries(allDebuffs)) {
        if (debuff.uniqueGroup === 210230 && debuff.source.skill?.classId === entityClassId) {
          const srcId = debuff.source.skill?.id ?? 0;
          if (identityBrandSourceSet.has(srcId)) {
            identityBrandDebuffIds.add(Number(idStr));
          } else {
            regularBrandDebuffIds.add(Number(idStr));
          }
        }
      }
      if (identityBrandDebuffIds.size === 0) continue;

      // Step 2: Sum window-damage across DPS party members for each brand type.
      const partyMembers =
        partiesList.length > 0
          ? (partiesList.find((party) => party.some((p) => p.name === name)) ?? this.playersOnly)
          : this.playersOnly;

      let identityBrandWindowDmg = 0;
      let regularBrandWindowDmg = 0;
      for (const player of partyMembers) {
        if (supportNames.has(player.name)) continue;
        for (const [idStr, dmg] of Object.entries(player.damageStats.debuffedBy)) {
          const id = Number(idStr);
          if (identityBrandDebuffIds.has(id)) identityBrandWindowDmg += dmg;
          else if (regularBrandDebuffIds.has(id)) regularBrandWindowDmg += dmg;
        }
      }
      if (identityBrandWindowDmg === 0 || regularBrandWindowDmg === 0) continue;

      // Step 3: Total regular brand bDMG from this support's skills (type 3 = boss debuff).
      const rawSkills = Object.values(entity.skills);
      let regularBrandBDmg = 0;
      for (const s of rawSkills) regularBrandBDmg += s.rdpsContributed[3] ?? 0;
      if (regularBrandBDmg === 0) continue;

      // Step 4: Extrapolate: how much brand bDMG was hidden in identity udpsContributed?
      // identityBrandBDmg = regularBrandBDmg * (identityBrandWindowDmg / regularBrandWindowDmg)
      const identityBrandBDmg = Math.round(
        regularBrandBDmg * (identityBrandWindowDmg / regularBrandWindowDmg)
      );
      if (identityBrandBDmg <= 0) continue;

      // Step 5: Identify which of this support's skills are identity skills by matching
      // their skill id against the known identity brand source list.
      const identitySkillIds = new Set<number>(supportSkills.identityBrandSources);
      let totalIdentityUdps = 0;
      let identityCasts = 0;
      for (const s of rawSkills) {
        if (!identitySkillIds.has(s.id)) continue;
        totalIdentityUdps += s.rdpsContributed[1] ?? 0;
        identityCasts += s.casts;
      }
      if (totalIdentityUdps === 0) continue;

      result.set(name, {
        bDmg: identityBrandBDmg,
        totalIdentityUdps,
        identitySkillIds,
        casts: identityCasts
      });
    }

    return result;
  });

  topDamageDealt = $derived(this.encounter?.encounterDamageStats.topDamageDealt ?? 0);

  contributionSplitByName = $derived.by(() => {
    const map = new Map<string, ContributionSplit>();
    const splits = this.encounter?.encounterDamageStats.misc?.contributionSplits;
    if (!splits) return map;
    for (const s of splits) map.set(s.name, s);
    return map;
  });

  sortValue(entity: Entity): number {
    switch (this.playerSort) {
      case "ndps":
        return getBaseDamage(entity.damageStats);
      case "rdps":
        return getRDamage(entity.damageStats);
      case "stagger":
        return entity.damageStats.stagger;
      default:
        return entity.damageStats.damageDealt;
    }
  }

  topSortValue = $derived.by(() => {
    const realPlayers = this.players.filter((p) => p.entityType === EntityType.PLAYER);
    if (!realPlayers.length) return 0;
    return Math.max(...realPlayers.map((p) => this.sortValue(p)));
  });

  /**
   * Array of damage dealt percentages for each player in the encounter, relative to the highest damage dealt.
   *
   * Indexes in the array correspond to the indexes in the players array.
   */
  playerDamagePercentages = $derived.by(() => {
    if (this.topSortValue === 0) return [];
    return this.players.map((player) => (this.sortValue(player) / this.topSortValue) * 100);
  });

  /**
   * Sum of all damage dealt by players in the encounter, including esthers if showEsther is enabled.
   */
  totalDamageDealt = $derived.by(() => {
    if (settings.app.general.showEsther) {
      // include esthers in the total damage dealt
      return (
        (this.encounter?.encounterDamageStats.totalDamageDealt ?? 0) +
        this.players
          .filter((e) => e.entityType === EntityType.ESTHER)
          .reduce((a, b) => a + b.damageStats.damageDealt, 0)
      );
    }

    return this.encounter?.encounterDamageStats.totalDamageDealt ?? 0;
  });

  /**
   * This dps is only used in live encounters, for logs it is obtained in encounterDamageStats
   */
  dps = $derived(this.duration <= 0 ? 0 : this.totalDamageDealt / (this.duration / 1000));

  /**
   * Array of players in the encounter, sorted by damage taken (descending).
   */
  playerDamageTakenSorted = $derived(
    this.players
      .filter((e) => e.damageStats.damageTaken > 0 && e.entityType === EntityType.PLAYER)
      .toSorted((a, b) => b.damageStats.damageTaken - a.damageStats.damageTaken)
  );

  /**
   * Array of damage taken percentages for each player in the encounter, relative to the highest damage taken.
   *
   * Indexes in the array correspond to the indexes in the playerDamageTakenSorted array.
   */
  playerDamageTakenPercentages = $derived.by(() => {
    if (!this.encounter || !this.encounter?.encounterDamageStats.topDamageTaken) return [];
    return this.playerDamageTakenSorted.map(
      (player) => (player.damageStats.damageTaken / this.encounter!.encounterDamageStats.topDamageTaken) * 100
    );
  });

  /**
   * Array of parties in the encounter, each party is an array of player names.
   *
   * e.g. [ ["Player1", "Player2", "Player3", "Player4"], ["Player5", "Player6", "Player7", "Player8"] ]
   */
  partyInfo: string[][] | undefined = $state(undefined);

  /**
   * Array of parties in the encounter, sorted by party order.
   *
   * Players in each party are sorted by damage dealt (descending).
   */
  parties = $derived.by(() => {
    if (!this.partyInfo) return [];

    // resolve map of id->name to array of player entities
    const temp = new Array<Array<Entity>>();

    this.partyInfo.forEach((party, partyId) => {
      temp[partyId] = party
        .map((name) => this.players.find((player) => player.name === name))
        .filter((player): player is Entity => player !== undefined);
    });

    // sort parties by partyId
    for (const party of temp) {
      switch (this.playerSort) {
        case "ndps":
          party.sort((a, b) => getBaseDamage(b.damageStats) - getBaseDamage(a.damageStats));
          break;
        case "rdps":
          party.sort((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
          break;
        case "stagger":
          party.sort((a, b) => b.damageStats.stagger - a.damageStats.stagger);
          break;
        default:
          party.sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
      }
    }

    return temp.length >= 1 ? temp : [this.players];
  });

  /**
   * Array of damage dealt percentages for each party in the encounter, relative to the highest damage dealt.
   *
   * Indexes in the array, and sub array correspond to the indexes in `this.parties`
   */
  partyDamagePercentages = $derived(
    this.parties.map((party) => party.map((player) => (this.sortValue(player) / this.topSortValue) * 100))
  );

  /**
   * Array of booleans to show if a party contains any dead players.
   */
  anyPartyDead = $derived(this.parties.map((x) => x.some((p) => p.isDead)));

  anySkillCastLog = $derived.by(() => {
    return this.players.some((player) => {
      return Object.entries(player.skills).some(([, skill]) => {
        return skill.skillCastLog && skill.skillCastLog.length > 0;
      });
    });
  });

  timeToKill = $derived.by(() => {
    if (this.duration < 0) {
      return undefined;
    }
    if (settings.app.meter.showTimeUntilKill && this.encounter?.currentBoss) {
      let remainingDpm =
        this.players
          .filter((e) => e.damageStats.damageDealt > 0 && !e.isDead && e.entityType == EntityType.PLAYER)
          .reduce((a, b) => a + b.damageStats.damageDealt, 0) / this.duration;
      let remainingBossHealth = this.encounter.currentBoss.currentHp + this.encounter.currentBoss.currentShield;
      let millisUntilKill = Math.max(remainingBossHealth / remainingDpm, 0);
      if (millisUntilKill > 3.6e6) {
        // 1 hr
        return "∞";
      } else {
        return timestampToMinutesAndSeconds(millisUntilKill);
      }
    }

    return undefined;
  });

  constructor(encounter?: Encounter, live: boolean = false) {
    this.encounter = encounter;
    if (encounter?.encounterDamageStats.misc?.partyInfo) {
      this.partyInfo = Object.values(encounter?.encounterDamageStats.misc?.partyInfo);
    }
    this.duration = encounter?.duration ?? 0;
    this.live = live;
  }

  reset() {
    this.encounter = undefined;
    this.duration = 0;
    this.partyInfo = undefined;
    this.playerSort = "dps";
  }
}
