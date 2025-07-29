import { abbreviateNumberSplit, customRound, formatPlayerName, getEstherFromNpcId } from "$lib/utils";
import { cardIds } from "./constants/cards";
import type { EncounterState } from "./encounter.svelte";
import { settings } from "./stores.svelte";
import { EntityType, type Entity, type IncapacitatedEvent } from "./types";
import { hyperAwakeningIds } from "./utils/buffs";

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

  skills = $derived.by(() => {
    if (!this.entity) return [];
    if (this.entity && this.entity.class === "Arcanist") {
      return Object.values(this.entity.skills)
        .sort((a, b) => b.totalDamage - a.totalDamage)
        .filter((skill) => !cardIds.includes(skill.id));
    } else {
      return Object.values(this.entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
    }
  });

  mostDamageSkill = $derived(this.skills[0]?.totalDamage ?? 0);

  skillDamagePercentages = $derived(this.skills.map((skill) => (skill.totalDamage / this.mostDamageSkill) * 100));
  anyBackAttacks = $derived(this.skills.some((skill) => skill.backAttacks > 0));
  anyFrontAttacks = $derived(this.skills.some((skill) => skill.frontAttacks > 0));
  anySupportBuff = $derived(this.skills.some((skill) => skill.buffedBySupport > 0));
  anySupportIdentity = $derived(this.skills.some((skill) => skill.buffedByIdentity > 0));
  anySupportBrand = $derived(this.skills.some((skill) => skill.debuffedBySupport > 0));
  anySupportHat = $derived(this.skills.some((skill) => skill.buffedByHat > 0));

  anyCooldownRatio = $derived(this.skills.some((skill) => skill.timeAvailable));

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
