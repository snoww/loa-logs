import { EntityType, type Encounter, type Entity, type PartyInfo } from "$lib/types";

export class EncounterState {
    paused = $state(false);
    live = false;

    encounter: Encounter | undefined = $state();

    settings: any = $state()!;
    colorStore: any = $state()!;
    curSettings = $derived(this.live ? this.settings.meter : this.settings.logs);
    end = $derived(this.encounter?.lastCombatPacket ?? 0);

    duration = $state(0);
    region = $derived.by(() => {
        const region = this.encounter?.encounterDamageStats.misc?.region ?? "";
        if (region && region === "EUC") {
            return "CE";
        }
        return region;
    });

    players = $derived.by(() => {
        if (!this.encounter || !this.encounter.entities) return [];
        if (this.settings.general.showEsther) {
            return Object.values(this.encounter.entities)
                .filter(
                    (e) =>
                        e.damageStats.damageDealt > 0 &&
                        (e.entityType === EntityType.ESTHER || (e.entityType === EntityType.PLAYER && e.classId != 0))
                )
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        } else {
            return Object.values(this.encounter.entities)
                .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId != 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        }
    });

    localPlayer = $derived(this.encounter?.localPlayer ?? "");

    bosses: Array<Entity> = $derived.by(() => {
        if (!this.encounter) return [];
        if (this.settings.general.showBosses) {
            return Object.values(this.encounter.entities)
                .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.BOSS)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        }
        return [];
    });

    isSolo = $derived(this.players.length === 1);
    anyDead = $derived(this.players.some((player) => player.isDead));
    multipleDeaths = $derived.by(() => {
        if (!this.anyDead) {
            return this.players.some((player) => player.damageStats.deaths > 0);
        } else {
            return this.players.some((player) => player.damageStats.deaths > 1);
        }
    });
    anyFrontAtk = $derived(this.players.some((player) => player.skillStats.frontAttacks > 0));
    anyBackAtk = $derived(this.players.some((player) => player.skillStats.backAttacks > 0));
    anySupportBuff = $derived(this.players.some((player) => player.damageStats.buffedBySupport > 0));
    anySupportIdentity = $derived(this.players.some((player) => player.damageStats.buffedByIdentity > 0));
    anySupportBrand = $derived(this.players.some((player) => player.damageStats.debuffedBySupport > 0));
    anySupportHat = $derived(
        this.players.some((player) => player.damageStats.buffedByHat && player.damageStats.buffedByHat > 0)
    );
    anyPlayerIncapacitated = $derived.by(() => {
        if (!this.encounter) return false;
        return Object.values(this.encounter.entities).some(
            (e) => e.damageStats.incapacitations && e.damageStats.incapacitations.length > 0
        );
    });

    topDamageDealt = $derived.by(() => {
        if (!this.encounter) return 0;
        return this.encounter.encounterDamageStats.topDamageDealt;
    });
    playerDamagePercentages = $derived.by(() => {
        if (this.topDamageDealt === 0) return [];
        return this.players.map((player) => (player.damageStats.damageDealt / this.topDamageDealt) * 100);
    });

    totalDamageDealt = $derived.by(() => {
        if (!this.encounter) return 0;
        if (this.settings.general.showEsther) {
            return (
                this.encounter.encounterDamageStats.totalDamageDealt +
                this.players
                    .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.ESTHER)
                    .reduce((a, b) => a + b.damageStats.damageDealt, 0)
            );
        } else {
            return this.encounter.encounterDamageStats.totalDamageDealt;
        }
    });

    playerDamageTakenSorted = $derived(
        this.players
            .filter((e) => e.damageStats.damageTaken > 0 && e.entityType === EntityType.PLAYER)
            .toSorted((a, b) => b.damageStats.damageTaken - a.damageStats.damageTaken)
    );
    playerDamageTakenPercentages = $derived.by(() => {
        if (!this.encounter?.encounterDamageStats.topDamageTaken) return [];
        return this.playerDamageTakenSorted.map(
            (player) => (player.damageStats.damageTaken / this.encounter!.encounterDamageStats.topDamageTaken) * 100
        );
    });

    partyInfo: PartyInfo | undefined = $state();

    anySkillCastLog = $derived.by(() => {
        return this.players.some((player) => {
            return Object.entries(player.skills).some(([, skill]) => {
                return skill.skillCastLog && skill.skillCastLog.length > 0;
            });
        });
    });

    constructor(encounter: Encounter | undefined, settings: any, live: boolean = false, colors: any) {
        this.live = live;
        this.encounter = encounter;
        this.duration = encounter?.duration ?? 0;
        this.partyInfo = encounter?.encounterDamageStats.misc?.partyInfo;
        this.settings = settings;
        this.colorStore = colors;
    }

    updateEncounter(encounter: Encounter) {
        this.encounter = encounter;
    }

    updateDuration(duration: number) {
        this.duration = duration;
    }

    updateSettings(settings: any) {
        this.settings = settings;
    }

    updatePartyInfo(partyInfo: any) {
        this.partyInfo = partyInfo;
    }

    pause() {
        this.paused = true;
    }

    reset() {
        this.encounter = undefined;
        this.duration = 0;
        this.partyInfo = undefined;
        this.paused = false;
    }
}
