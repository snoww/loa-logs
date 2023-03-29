export interface EncounterEvent {
    event: string;
    payload: Encounter;
}

export interface Encounter {
    lastCombatPacket: number;
    fightStart: number;
    localPlayer: string;
    entities: {[key: string]: Entity};
    currentBoss?: Entity;
    encounterDamageStats: EncounterDamageStats;
    duration: number;
    reset: boolean;
}

export interface EncounterDamageStats {
    totalDamageDealt: number,
    topDamageDealt: number,
    totalDamageTaken: number,
    topDamageTaken: number,
    dps: number,
    dpsIntervals: { [key: number]: number },
    mostDamageTakenEntity: MostDamageTakenEntity
}

export interface MostDamageTakenEntity {
    name: string,
    damageTaken: number
}

export interface Entity {
    lastUpdate: number;
    id: string;
    npcId: number;
    name: string;
    entityType: EntityType;
    classId: number;
    class: string;
    gearScore: number;
    currentHp: number;
    maxHp: number;
    isDead: boolean;
    skills: {[skillName: string]: Skill};
    damageStats: DamageStats;
    skillStats: SkillStats;
}

export interface Skill {
    id: number;
    name: string;
    totalDamage: number;
    maxDamage: number;
    casts: number;
    hits: number;
    crits: number;
    backAttacks: number;
    frontAttacks: number;
    dps: number;
    dpsIntervals: { [key: number]: number };
}

export interface DamageStats {
    damageDealt: number;
    damageTaken: number;
    deaths: number;
    deathTime: number;
    dps: number;
    dpsIntervals: { [key: number]: number };
}

interface SkillStats {
    casts: number;
    hits: number;
    crits: number;
    backAttacks: number;
    frontAttacks: number;
    counters: number;
}

export enum EntityType {
    UNKNOWN,
    MONSTER,
    BOSS,
    GUARDIAN,
    PLAYER,
    NPC
}

export interface ClassColors {
    [key: string]: {
        color: string,
        defaultColor: string
    }
}

export interface bossMap {
    [key: string]: number;
}
