import type { EncounterState } from "./encounter.svelte";
import {
  Buff,
  BuffDetails,
  EntityType,
  Shield,
  ShieldDetails,
  ShieldTab,
  type Entity,
  type MeterTab,
  type StatusEffect
} from "./types";
import { addBardBubbles, filterStatusEffects, supportSkills } from "./utils/buffs";

import { customRound } from "$lib/utils";

export class BuffState {
  enc: EncounterState = $state()!;
  focusedPlayer: Entity | undefined = $state();
  tab: MeterTab | undefined = $state();

  players = $derived.by(() => {
    if (!this.enc) return [];
    return this.enc.players.filter((player) => player.entityType === EntityType.PLAYER);
  });
  percentages = $derived(
    this.players.map((player) => (player.damageStats.damageDealt / this.enc.topDamageDealt) * 100)
  );

  groupedSynergies: Map<string, Map<number, StatusEffect>> = $derived.by(() => {
    if (!this.enc.encounter) return new Map();
    const temp = new Map();
    for (const [id, buff] of Object.entries(this.enc.encounter.encounterDamageStats.buffs)) {
      if (this.focusedPlayer && !Object.hasOwn(this.focusedPlayer.damageStats.buffedBy, id)) {
        continue;
      }
      filterStatusEffects(temp, buff, Number(id), this.focusedPlayer, this.tab);
    }
    for (const [id, debuff] of Object.entries(this.enc.encounter.encounterDamageStats.debuffs)) {
      if (this.focusedPlayer && !Object.hasOwn(this.focusedPlayer.damageStats.debuffedBy, id)) {
        continue;
      }
      filterStatusEffects(temp, debuff, Number(id), this.focusedPlayer, this.tab);
    }

    return new Map([...temp.entries()].sort());
  });

  partyPercentages = $derived(
    this.enc.parties.map((party) =>
      party.map((player) => (player.damageStats.damageDealt / this.enc.topDamageDealt) * 100)
    )
  );

  partyGroupedSynergies: Map<string, Set<string>> = $derived.by(() => {
    if (this.groupedSynergies.size === 0 || this.enc.parties.length < 1) return new Map();
    const temp = new Map();
    this.enc.parties.forEach((party, partyId) => {
      temp.set(partyId.toString(), new Set<string>());
      const partySyns = new Set<string>();
      for (const player of party) {
        this.groupedSynergies.forEach((synergies, key) => {
          synergies.forEach((_, id) => {
            if (player.damageStats.buffedBy[id] || player.damageStats.debuffedBy[id]) {
              partySyns.add(key);
            }
          });
        });
      }
      temp.set(partyId.toString(), new Set([...partySyns].sort()));
    });

    return temp;
  });

  partyBuffs: Map<string, Map<string, Array<BuffDetails>>> = $derived.by(() => {
    const temp: Map<string, Map<string, Array<BuffDetails>>> = new Map();
    this.enc.parties.forEach((party, partyId) => {
      temp.set(partyId.toString(), new Map<string, Array<BuffDetails>>());
      for (const player of party) {
        temp.get(partyId.toString())!.set(player.name, []);
        const playerBuffs = temp.get(partyId.toString())!.get(player.name)!;

        const damageDealtWithoutSpecial =
          player.damageStats.damageDealt -
          Object.values(player.skills)
            .filter((skill) => skill.special)
            .reduce((acc, skill) => acc + skill.totalDamage, 0);
        const damageDealtWithoutSpecialAndHa =
          damageDealtWithoutSpecial - (player.damageStats.hyperAwakeningDamage ?? 0);

        this.partyGroupedSynergies.get(partyId.toString())?.forEach((key) => {
          const buffDetails = new BuffDetails();
          buffDetails.id = key;
          let buffDamage = 0;
          const buffs = this.groupedSynergies.get(key) || new Map();
          let isHat = false;

          buffs.forEach((syn, id) => {
            if (supportSkills.haTechnique.includes(id)) {
              isHat = true;
            }

            if (player.damageStats.buffedBy[id] && syn.category === "buff") {
              const b = new Buff(
                syn.source.icon,
                customRound(
                  (player.damageStats.buffedBy[id] /
                    (isHat ? damageDealtWithoutSpecial : damageDealtWithoutSpecialAndHa)) *
                    100
                ),
                syn.source.skill?.icon
              );
              addBardBubbles(key, b, syn);
              buffDetails.buffs.push(b);
              buffDamage += player.damageStats.buffedBy[id];
            }
            if (player.damageStats.debuffedBy[id] && syn.category === "debuff") {
              buffDetails.buffs.push(
                new Buff(
                  syn.source.icon,
                  customRound(
                    (player.damageStats.debuffedBy[id] /
                      (isHat ? damageDealtWithoutSpecial : damageDealtWithoutSpecialAndHa)) *
                      100
                  ),
                  syn.source.skill?.icon
                )
              );
              buffDamage += player.damageStats.debuffedBy[id];
            }
          });
          if (buffDamage > 0) {
            buffDetails.percentage = customRound(
              (buffDamage / (isHat ? damageDealtWithoutSpecial : damageDealtWithoutSpecialAndHa)) * 100
            );
          }

          playerBuffs.push(buffDetails);
        });
      }
    });

    return temp;
  });

  shieldBy = $state("");
  shieldValue = $state("");
  topShield = $derived(Math.max(...this.players.map((player) => player.damageStats[this.shieldValue])));

  groupedShields: Map<string, Map<number, StatusEffect>> = $derived.by(() => {
    if (!this.enc.encounter) return new Map();
    const temp = new Map();
    for (const [id, shield] of Object.entries(this.enc.encounter.encounterDamageStats.appliedShieldBuffs)) {
      filterStatusEffects(temp, shield, Number(id), undefined, undefined, true);
    }

    return new Map([...temp.entries()].sort());
  });

  shieldParties = $derived.by(() => {
    if (!this.enc.partyInfo) return [];

    const temp: Entity[][] = [];

    this.enc.partyInfo.forEach((party, partyId) => {
      temp[partyId] = party
        .map((name) => this.players.find((player) => player.name === name))
        .filter((player): player is Entity => player !== undefined);
    });

    temp.forEach((party) => {
      if (party.length > 0) {
        party.sort((a, b) => b.damageStats[this.shieldValue] - a.damageStats[this.shieldValue]);
      }
    });

    return temp.length >= 2
      ? temp
      : [this.players.toSorted((a, b) => b.damageStats[this.shieldValue] - a.damageStats[this.shieldValue])];
  });

  shieldPartyPercentages = $derived(
    this.shieldParties.map((party) =>
      party.map((player) => (player.damageStats[this.shieldValue] / this.topShield) * 100)
    )
  );

  partyGroupedShields: Map<string, Set<string>> = $derived.by(() => {
    if (this.groupedShields.size === 0 || this.shieldParties.length < 1) return new Map();
    const temp = new Map();
    this.shieldParties.forEach((party, partyId) => {
      temp.set(partyId.toString(), new Set<string>());
      const partyShields = new Set<string>();
      for (const player of party) {
        this.groupedShields.forEach((shields, key) => {
          shields.forEach((_, id) => {
            if (player.damageStats[this.shieldBy][id]) {
              partyShields.add(key);
            }
          });
        });
      }
      temp.set(partyId.toString(), new Set([...partyShields].sort()));
    });

    return temp;
  });

  partyShields: Map<string, Map<string, Array<ShieldDetails>>> = $derived.by(() => {
    const temp: Map<string, Map<string, Array<ShieldDetails>>> = new Map();
    this.shieldParties.forEach((party, partyId) => {
      temp.set(partyId.toString(), new Map<string, Array<ShieldDetails>>());
      for (const player of party) {
        temp.get(partyId.toString())!.set(player.name, []);
        const playerShields = temp.get(partyId.toString())!.get(player.name)!;
        this.partyGroupedShields.get(partyId.toString())?.forEach((key) => {
          const shieldDetails = new ShieldDetails();
          shieldDetails.id = key;
          let shieldTotal = 0;
          const buffs = this.groupedShields.get(key) || new Map();
          buffs.forEach((syn, id) => {
            if (player.damageStats[this.shieldBy][id]) {
              const s = new Shield(id, syn.source.icon, player.damageStats[this.shieldBy][id]);
              shieldDetails.buffs.push(s);
              shieldTotal += player.damageStats[this.shieldBy][id];
            }
          });
          shieldDetails.total = shieldTotal;
          playerShields.push(shieldDetails);
        });
      }
    });

    return temp;
  });

  constructor(enc: EncounterState) {
    this.enc = enc;
  }

  setFocusedPlayer(player: Entity | undefined) {
    this.focusedPlayer = player;
  }

  setTab(tab: MeterTab | undefined) {
    this.tab = tab;
  }

  setShieldTab(shieldTab: ShieldTab | undefined) {
    switch (shieldTab) {
      case ShieldTab.GIVEN:
        this.shieldBy = "shieldsGivenBy";
        this.shieldValue = "shieldsGiven";
        break;
      case ShieldTab.RECEIVED:
        this.shieldBy = "shieldsReceivedBy";
        this.shieldValue = "shieldsReceived";
        break;
      case ShieldTab.E_GIVEN:
        this.shieldBy = "damageAbsorbedOnOthersBy";
        this.shieldValue = "damageAbsorbedOnOthers";
        break;
      case ShieldTab.E_RECEIVED:
        this.shieldBy = "damageAbsorbedBy";
        this.shieldValue = "damageAbsorbed";
        break;
    }
  }
}
