import { classesMap } from "$lib/constants/classes";
import { type Encounter, type Entity, EntityType } from "$lib/types";

export function createPreviewEncounter(playerCount: 4 | 8): Encounter {
  const duration = 120000;
  const classIds = [102, 205, 402, 204, 112, 512, 313, 602];
  const players: Entity[] = classIds.slice(0, playerCount).map((classId, index) => {
    const isSupport = index % 4 === 3;
    const dps = isSupport ? 2000000 : 100000000 - index * 8000000;
    const damageDealt = dps * (duration / 1000);
    return {
      lastUpdate: duration,
      id: index + 1,
      npcId: 0,
      name: `Party ${Math.floor(index / 4) + 1} ${isSupport ? "Support" : `DPS ${(index % 4) + 1}`}`,
      characterId: String(index + 1),
      entityType: EntityType.PLAYER,
      classId,
      class: classesMap[classId],
      gearScore: 1700,
      currentHp: 200000,
      maxHp: 200000,
      currentShield: 0,
      isDead: false,
      skills: {},
      damageStats: {
        damageDealt,
        damageTaken: 0,
        buffedBy: {},
        debuffedBy: {},
        buffedBySupport: damageDealt * 0.95,
        buffedByIdentity: damageDealt * 0.7,
        debuffedBySupport: damageDealt * 0.98,
        backAttackDamage: 0,
        frontAttackDamage: 0,
        critDamage: 0,
        shieldsGiven: 0,
        shieldsReceived: 0,
        damageAbsorbed: 0,
        damageAbsorbedOnOthers: 0,
        shieldsGivenBy: {},
        shieldsReceivedBy: {},
        damageAbsorbedBy: {},
        damageAbsorbedOnOthersBy: {},
        deaths: 0,
        deathTime: 0,
        dps,
        dpsAverage: [0, dps],
        dpsRolling10sAvg: [0, dps],
        rdpsDamageReceived: 0,
        rdpsDamageReceivedSupport: 0,
        rdpsDamageGiven: 0,
        incapacitations: [],
        stagger: 0,
        unbuffedDamage: damageDealt,
        unbuffedDps: dps
      },
      skillStats: {
        casts: 0,
        hits: 0,
        crits: 0,
        backAttacks: 0,
        frontAttacks: 0,
        counters: 0
      }
    };
  });
  const totalDamageDealt = players.reduce((total, player) => total + player.damageStats.damageDealt, 0);

  return {
    lastCombatPacket: duration,
    fightStart: 0,
    localPlayer: "",
    entities: Object.fromEntries(players.map((player) => [player.name, player])),
    currentBossName: "",
    currentBoss: null,
    encounterDamageStats: {
      totalDamageDealt,
      topDamageDealt: players[0].damageStats.damageDealt,
      totalDamageTaken: 0,
      topDamageTaken: 0,
      dps: totalDamageDealt / (duration / 1000),
      mostDamageTakenEntity: { name: "", damageTaken: 0 },
      buffs: {},
      debuffs: {},
      totalShielding: 0,
      totalEffectiveShielding: 0,
      appliedShieldBuffs: {},
      bossHpLog: {},
      misc: {
        partyInfo: Object.fromEntries(
          Array.from({ length: playerCount / 4 }, (_, party) => [
            String(party),
            players.slice(party * 4, party * 4 + 4).map((player) => player.name)
          ])
        )
      }
    },
    duration,
    reset: false,
    favorite: false,
    cleared: false,
    bossOnlyDamage: true
  };
}
