import { type Encounter, type Entity, EntityType } from "$lib/types";
import { settings } from "./stores.svelte";
import { timestampToMinutesAndSeconds } from "./utils";

export class EncounterState {
  live = false;

  encounter: Encounter | undefined = $state();
  curSettings = $derived(this.live ? settings.app.meter : settings.app.logs);
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
    return Object.values(this.encounter.entities)
      .filter((e) => {
        if (e.damageStats.damageDealt <= 0) return false;

        const isValidPlayer = e.entityType === EntityType.PLAYER && e.classId !== 0;
        if (settings.app.general.showEsther) {
          return isValidPlayer || e.entityType === EntityType.ESTHER;
        }
        return isValidPlayer;
      })
      .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
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

  // this section is used for conditional headers in the damage table
  isSolo = $derived(this.players.length === 1);
  anyDead = $derived(this.players.some((player) => player.isDead));
  multipleDeaths = $derived(this.players.some((player) => player.damageStats.deaths > 0 && !player.isDead));
  anyFrontAtk = $derived(this.players.some((player) => player.skillStats.frontAttacks > 0));
  anyBackAtk = $derived(this.players.some((player) => player.skillStats.backAttacks > 0));
  anySupportBuff = $derived(this.players.some((player) => player.damageStats.buffedBySupport > 0));
  anySupportIdentity = $derived(this.players.some((player) => player.damageStats.buffedByIdentity > 0));
  anySupportBrand = $derived(this.players.some((player) => player.damageStats.debuffedBySupport > 0));
  anySupportHat = $derived(
    this.players.some((player) => player.damageStats.buffedByHat && player.damageStats.buffedByHat > 0)
  );
  anyPlayerIncapacitated = $derived(
    this.players.some((player) => player.damageStats.incapacitations && player.damageStats.incapacitations.length > 0)
  );
  topDamageDealt = $derived(this.encounter?.encounterDamageStats.topDamageDealt ?? 0);

  /**
   * Array of damage dealt percentages for each player in the encounter, relative to the highest damage dealt.
   *
   * Indexes in the array correspond to the indexes in the players array.
   */
  playerDamagePercentages = $derived.by(() => {
    if (this.topDamageDealt === 0) return [];
    return this.players.map((player) => (player.damageStats.damageDealt / this.topDamageDealt) * 100);
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
      party.sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
    }

    return temp.length >= 1 ? temp : [this.players];
  });

  /**
   * Array of damage dealt percentages for each party in the encounter, relative to the highest damage dealt.
   *
   * Indexes in the array, and sub array correspond to the indexes in `this.parties`
   */
  partyDamagePercentages = $derived(
    this.parties.map((party) => party.map((player) => (player.damageStats.damageDealt / this.topDamageDealt) * 100))
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
  }
}
