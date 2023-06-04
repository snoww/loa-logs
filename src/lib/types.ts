export interface EncounterEvent {
    event: string;
    payload: Encounter;
}

export interface Encounter {
    lastCombatPacket: number;
    fightStart: number;
    localPlayer: string;
    entities: { [key: string]: Entity };
    currentBossName: string;
    currentBoss: Entity | null;
    encounterDamageStats: EncounterDamageStats;
    duration: number;
    reset: boolean;
}

export interface EncountersOverview {
    encounters: Array<EncounterPreview>;
    totalEncounters: number;
}

export interface EncounterPreview {
    id: number;
    fightStart: number;
    bossName: string;
    duration: number;
    classes: Array<number>;
    names: Array<string>;
}

export interface EncounterDamageStats {
    totalDamageDealt: number;
    topDamageDealt: number;
    totalDamageTaken: number;
    topDamageTaken: number;
    dps: number;
    dpsIntervals: { [key: number]: number };
    mostDamageTakenEntity: MostDamageTakenEntity;
    buffs: { [key: number]: StatusEffect };
    debuffs: { [key: number]: StatusEffect };
    misc?: EncounterMisc;
}

export interface EncounterMisc {
    staggerStats: StaggerStats;
    bossHpLog: { [key: string]: Array<BossHpLog> };
}

export class BossHpLog {
    time: number;
    hp: number;
    p: number;

    constructor(time: number, hp: number, p: number) {
        this.time = time;
        this.hp = hp;
        this.p = p;
    }
}

export interface StaggerStats {
    log: Array<[number, number]>;
    average: number;
    staggersPerMin: number;
}

export interface MostDamageTakenEntity {
    name: string;
    damageTaken: number;
}

export interface Entity {
    lastUpdate: number;
    id: number;
    npcId: number;
    name: string;
    entityType: EntityType;
    classId: number;
    class: string;
    gearScore: number;
    currentHp: number;
    maxHp: number;
    isDead: boolean;
    skills: { [skillId: number]: Skill };
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
    castLog: Array<number>;
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
    dpsAverage: [number, number];
    dpsRolling10sAvg: [number, number];
}

export interface SkillStats {
    casts: number;
    hits: number;
    crits: number;
    backAttacks: number;
    frontAttacks: number;
    counters: number;
    identityStats?: string;
}

export type IdentityLogTypeValue = number | [number, number] | [number, number, number];
export type IdentityLogType = Array<[number, IdentityLogTypeValue]>;

export interface IdentityStats {
    log: IdentityLogType;
    average: number;
    cardDraws?: { [key: number]: number };
}

export interface StatusEffect {
    [x: string]: any;
    target: StatusEffectTarget;
    category: string;
    buffCategory: string;
    buffType: number;
    uniqueGroup: number;
    source: StatusEffectSource;
}

export enum StatusEffectTarget {
    OTHER = "OTHER",
    SELF = "SELF",
    PARTY = "PARTY"
}

export interface StatusEffectSource {
    name: string;
    desc: string;
    icon: string;
    skill: SkillData | null;
    setName: string | null;
}

export interface SkillData {
    id: number;
    name: string;
    desc: string;
    classId: number;
    icon: string;
    summonIds: Array<number> | null;
    summonSourceSkill: Array<number> | null;
    sourceSkill: number | null;
}

export enum EntityType {
    UNKNOWN = "UNKNOWN",
    MONSTER = "MONSTER",
    BOSS = "BOSS",
    GUARDIAN = "GUARDIAN",
    PLAYER = "PLAYER",
    NPC = "NPC",
    ESTHER = "ESTHER"
}

export interface ClassColors {
    [key: string]: {
        color: string;
        defaultColor: string;
    };
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
    IDENTITY,
    STAGGER
}

export enum ChartType {
    AVERAGE_DPS,
    ROLLING_DPS,
    SKILL_LOG,
    IDENTITY,
    STAGGER
}

export interface ClassMap {
    [key: number]: string;
}

export enum StatusEffectBuffTypeFlags {
    NONE = 0,
    DMG = 1,
    CRIT = 2,
    ATKSPEED = 4,
    MOVESPEED = 8,
    HP = 16,
    DEFENSE = 32,
    RESOURCE = 64,
    COOLDOWN = 128,
    STAGGER = 256,
    SHIELD = 512,
    ANY = 262144
}

export class BuffDetails {
    percentage: string;
    buffs: Array<Buff>;
    id: string;

    constructor() {
        this.percentage = "";
        this.buffs = [];
        this.id = "";
    }
}

export class Buff {
    icon: string;
    sourceIcon?: string;
    percentage: string;

    constructor(icon: string, percentage: string, sourceIcon?: string) {
        this.icon = icon;
        this.sourceIcon = sourceIcon;
        this.percentage = percentage;
    }
}
