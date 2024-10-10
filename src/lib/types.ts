export interface EncounterEvent {
    event: string;
    payload: Encounter;
}

export interface PartyEvent {
    event: string;
    payload?: PartyInfo;
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
    difficulty?: string;
    favorite: boolean;
    cleared: boolean;
    bossOnlyDamage: boolean;
    sync?: string;
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
    difficulty?: string;
    localPlayer: string;
    myDps: number;
    favorite: boolean;
    cleared: boolean;
}

export interface EncounterDamageStats {
    totalDamageDealt: number;
    topDamageDealt: number;
    totalDamageTaken: number;
    topDamageTaken: number;
    dps: number;
    mostDamageTakenEntity: MostDamageTakenEntity;
    buffs: { [key: number]: StatusEffect };
    debuffs: { [key: number]: StatusEffect };
    totalShielding: number;
    totalEffectiveShielding: number;
    appliedShieldBuffs: { [key: number]: StatusEffect };
    misc?: EncounterMisc;
    bossHpLog: { [key: string]: Array<BossHpLog> };
    staggerStats?: StaggerStats;
}

export interface EncounterMisc {
    staggerStats?: StaggerStats;
    bossHpLog?: { [key: string]: Array<BossHpLog> };
    partyInfo?: PartyInfo;
    rdpsValid?: boolean;
    rdpsMessage?: string;
}

export interface PartyInfo {
    [key: string]: Array<string>;
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
    currentShield: number;
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
    maxDamageCast: number;
    buffedBy: { [key: number]: number };
    debuffedBy: { [key: number]: number };
    buffedBySupport: number;
    buffedByIdentity: number;
    buffedByHat: number;
    debuffedBySupport: number;
    casts: number;
    hits: number;
    crits: number;
    critDamage: number;
    backAttacks: number;
    frontAttacks: number;
    backAttackDamage: number;
    frontAttackDamage: number;
    dps: number;
    castLog: Array<number>;
    tripodIndex?: Tripod;
    tripodLevel?: Tripod;
    gemCooldown?: number;
    gemDamage?: number;
    rdpsDamageReceived: number;
    rdpsDamageReceivedSupport: number;
    rdpsDamageGiven: number;
    skillCastLog: Array<SkillCast>;
}

export interface SkillCast {
    timestamp: number;
    last: number;
    hits: SkillHit[];
}

export interface SkillHit {
    timestamp: number;
    damage: number;
    crit: boolean;
    backAttack: boolean;
    frontAttack: boolean;
    buffedBy: number[];
    debuffedBy: number[];
    rdpsDamageReceived: number;
    rdpsDamageReceivedSupport: number;
}

export interface Tripod {
    first: number;
    second: number;
    third: number;
}

export interface DamageStats {
    damageDealt: number;
    damageTaken: number;
    hyperAwakeningDamage?: number;
    buffedBy: { [key: number]: number };
    debuffedBy: { [key: number]: number };
    buffedBySupport: number;
    buffedByIdentity: number;
    buffedByHat?: number;
    debuffedBySupport: number;
    backAttackDamage: number;
    frontAttackDamage: number;
    critDamage: number;
    shieldsGiven: number;
    shieldsReceived: number;
    damageAbsorbed: number;
    damageAbsorbedOnOthers: number;
    shieldsGivenBy: { [key: number]: number };
    shieldsReceivedBy: { [key: number]: number };
    damageAbsorbedBy: { [key: number]: number };
    damageAbsorbedOnOthersBy: { [key: number]: number };
    deaths: number;
    deathTime: number;
    dps: number;
    dpsAverage: [number, number];
    dpsRolling10sAvg: [number, number];
    rdpsDamageReceived: number;
    rdpsDamageReceivedSupport: number;
    rdpsDamageGiven: number;
    [key: string]: any;
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

export interface StatusEffectWithId {
    id: number;
    statusEffect: StatusEffect;
}

export interface SkillChartSupportDamage {
    buff: number;
    brand: number;
    identity: number;
}

export interface SkillChartModInfo {
    crit: number;
    critDamage: number;
    ba: number;
    fa: number;
}

export interface SkillCastInfo {
    skillId: number;
    cast: number;
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

export interface IdentityEvent {
    gauge1: number;
    gauge2: number;
    gauge3: number;
}

export interface StaggerEvent {
    current: number;
    max: number;
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
    RDPS,
    TANK,
    PARTY_BUFFS,
    SELF_BUFFS,
    IDENTITY,
    STAGGER,
    DETAILS,
    BOSS,
    SHIELDS
}

export enum ChartType {
    AVERAGE_DPS,
    ROLLING_DPS,
    SKILL_LOG,
    IDENTITY,
    STAGGER
}

export enum ShieldTab {
    GIVEN,
    RECEIVED,
    E_GIVEN,
    E_RECEIVED
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
    bonus?: number;

    constructor(icon: string, percentage: string, sourceIcon?: string) {
        this.icon = icon;
        this.sourceIcon = sourceIcon;
        this.percentage = percentage;
    }
}

export class ShieldDetails {
    total: number;
    buffs: Array<Shield>;
    id: string;

    constructor() {
        this.total = 0;
        this.buffs = [];
        this.id = "";
    }
}

export class Shield {
    id: number;
    icon: string;
    value: number;

    constructor(id: number, icon: string, value: number) {
        this.id = id;
        this.icon = icon;
        this.value = value;
    }
}

export class MiniSkill {
    name: string;
    icon: string;
    castLog: Array<number>;

    constructor(name: string, icon: string, castLog: Array<number>) {
        this.name = name;
        this.icon = icon;
        this.castLog = castLog;
    }
}

export class OpenerSkill {
    name: string;
    icon: string;

    constructor(name: string, icon: string) {
        this.name = name;
        this.icon = icon;
    }
}

export interface EncounterDbInfo {
    size: string;
    totalEncounters: number;
    totalEncountersFiltered: number;
}

export class SearchFilter {
    bosses: Set<string>;
    encounters: Set<string>;
    minDuration: number;
    favorite: boolean;
    cleared: boolean;
    difficulty: string;
    bossOnlyDamage: boolean;
    sort: string;
    order: number;

    constructor(minDuration = -1) {
        this.bosses = new Set();
        this.encounters = new Set();
        this.minDuration = minDuration;
        this.favorite = false;
        this.cleared = false;
        this.difficulty = "";
        this.bossOnlyDamage = false;
        this.sort = "id";
        this.order = 2;
    }
}

export interface PartyBuffs {
    parties: Array<Array<Entity>>;
    partyGroupedSynergies: Map<string, Set<string>>;
    partyPercentages: Array<number[]>;
    partyBuffs: Map<string, Map<string, Array<BuffDetails>>>;
}
