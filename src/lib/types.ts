export interface EncounterEvent {
    event: string;
    payload: Encounter;
}

export interface Encounter {
    lastCombatPacket: number;
    fightStart: number;
    localPlayer: string;
    entities: {[key: string]: Entity};
    currentBossName: string;
    currentBoss: Entity | null;
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
    mostDamageTakenEntity: MostDamageTakenEntity,
    buffs: { [key: number]: StatusEffect },
    debuffs: { [key: number]: StatusEffect },
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
    skills: {[skillId: number]: Skill};
    damageStats: DamageStats;
    skillStats: SkillStats;
}

export interface Skill {
    id: number;
    name: string;
    icon: string;
    totalDamage: number;
    maxDamage: number;
    buffedBy: { [key: number]: number };
    debuffedBy: { [key: number]: number };
    buffedBySupport: number;
    debuffedBySupport: number;
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
    buffedBy: { [key: number]: number };
    debuffedBy: { [key: number]: number };
    buffedBySupport: number;
    debuffedBySupport: number;
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

export interface StatusEffect {
    [x: string]: any;
    target: StatusEffectTarget;
    category: string,
    buffCategory: string,
    buffType: number,
    uniqueGroup: number,
    source: StatusEffectSource
}

export enum StatusEffectTarget {
    OTHER = "OTHER",
    SELF = "SELF",
    PARTY = "PARTY",
}

export interface StatusEffectSource {
    name: string,
    desc: string,
    icon: string,
    skill: SkillData | null,
    set_name: string | null
}

export interface SkillData {
    id: number,
    name: string,
    desc: string,
    classId: number,
    icon: string,
    summonIds: Array<number> | null,
    summonSourceSkill: Array<number> | null,
    sourceSkill: number | null,
}

export enum EntityType {
    UNKNOWN = "UNKNOWN",
    MONSTER = "MONSTER",
    BOSS = "BOSS",
    GUARDIAN = "GUARDIAN",
    PLAYER = "PLAYER",
    NPC = "NPC",
}

export interface ClassColors {
    [key: string]: {
        color: string,
        defaultColor: string
    }
}

export interface BossMap {
    [key: string]: number;
}

export enum MeterState {
    PARTY,
    PLAYER
}

export enum MeterTab {
    DAMAGE,
    TANK,
    PARTY_BUFFS,
    SELF_BUFFS,
}

export interface ClassMap {
    [key: number]: string;
}