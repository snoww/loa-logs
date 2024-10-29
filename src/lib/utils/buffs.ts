import { classesMap, classNameToClassId } from "$lib/constants/classes";
import {
    StatusEffectBuffTypeFlags,
    type StatusEffect,
    type Entity,
    StatusEffectTarget,
    MeterTab,
    Buff,
    BuffDetails,
    type Skill,
    type PartyBuffs,
    type PartyInfo,
    ShieldTab,
    Shield,
    ShieldDetails,
    type EncounterDamageStats,
    type StatusEffectWithId,
    type SkillChartSupportDamage,
    type DamageStats
} from "$lib/types";
import { round } from "./numbers";
import { getSkillIcon } from "./strings";

export function defaultBuffFilter(buffType: number): boolean {
    return (
        ((StatusEffectBuffTypeFlags.DMG |
            StatusEffectBuffTypeFlags.CRIT |
            StatusEffectBuffTypeFlags.ATKSPEED |
            StatusEffectBuffTypeFlags.MOVESPEED |
            StatusEffectBuffTypeFlags.COOLDOWN) &
            buffType) !==
        0
    );
}

export function groupedSynergiesAdd(
    map: Map<string, Map<number, StatusEffect>>,
    key: string,
    id: number,
    buff: StatusEffect,
    focusedPlayer: Entity | undefined,
    buffFilter = true
) {
    // by default, only show dmg, crit, atk spd, cd buffs.
    // show all arcana cards for fun
    if (!focusedPlayer || focusedPlayer.classId !== 202) {
        if (buffFilter && !defaultBuffFilter(buff.buffType)) {
            return;
        }
    }
    key = key.replaceAll(" ", "").toLowerCase();
    if (map.has(key)) {
        map.get(key)?.set(id, buff);
    } else {
        map.set(key, new Map([[id, buff]]));
    }
}

export function filterStatusEffects(
    groupedSynergies: Map<string, Map<number, StatusEffect>>,
    buff: StatusEffect,
    id: number,
    focusedPlayer: Entity | undefined,
    tab: MeterTab | null,
    buffFilter = true,
    shields = false
) {
    let key = "";

    // Party synergies
    if (isPartySynergy(buff)) {
        if (tab !== MeterTab.PARTY_BUFFS && !shields) {
            return;
        }

        if (isSupportBuff(buff)) {
            key = makeSupportBuffKey(buff);
        } else {
            key = `${classesMap[buff.source.skill?.classId ?? 0]}_${
                buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
            }`;
        }

        groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
    }
    // Self synergies
    else if (isSelfItemSynergy(buff)) {
        if (tab !== MeterTab.SELF_BUFFS && !shields) {
            return;
        }

        if (buff.buffCategory === "bracelet") {
            // put bracelets buffs at the end
            key = `zzbracelet_${buff.uniqueGroup}`;
        } else if (buff.buffCategory === "elixir") {
            key = `elixir_${buff.uniqueGroup}`;
        } else {
            key = buff.buffCategory;
        }
        groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
    }
    // set synergies
    else if (isSetSynergy(buff)) {
        if ((tab === MeterTab.SELF_BUFFS && !focusedPlayer) || shields) {
            // put set buffs at the start
            groupedSynergiesAdd(groupedSynergies, `_set_${buff.source.setName}`, id, buff, focusedPlayer, buffFilter);
        }
    }
    // self & other identity, class skill, engravings
    else if (isSelfSkillSynergy(buff)) {
        if (tab === MeterTab.SELF_BUFFS && focusedPlayer) {
            if (buff.buffCategory === "ability") {
                key = `${buff.uniqueGroup ? buff.uniqueGroup : id}`;
            } else {
                if (focusedPlayer.classId !== buff.source.skill?.classId) {
                    return; // We hide other classes self buffs (class_skill & identity)
                }
                key = `_${classesMap[buff.source.skill?.classId ?? 0]}_${
                    buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
                }`;
            }
            groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
        } else if (shields) {
            if (isSupportBuff(buff)) {
                key = makeSupportBuffKey(buff);
            } else if (buff.buffCategory === "ability") {
                key += `${buff.uniqueGroup ? buff.uniqueGroup : id}`;
            } else {
                key += `_${classesMap[buff.source.skill?.classId ?? 0]}_${
                    buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
                }`;
            }
            groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
        }
    }
    // other synergies
    else if (isOtherSynergy(buff)) {
        if ((tab === MeterTab.SELF_BUFFS && focusedPlayer) || shields) {
            groupedSynergiesAdd(groupedSynergies, `etc_${buff.source.name}`, id, buff, focusedPlayer, buffFilter);
        }
    }
}

export function getSynergyPercentageDetails(groupedSynergies: Map<string, Map<number, StatusEffect>>, skill: Skill) {
    const synergyPercentageDetails: BuffDetails[] = [];
    const isHyperAwakening = hyperAwakeningIds.has(skill.id);
    groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        const buff = new BuffDetails();
        buff.id = key;

        synergies.forEach((syn, id) => {
            if (isHyperAwakening) {
                if (supportSkills.haTechnique.includes(id)) {
                    const b = new Buff(
                        syn.source.icon,
                        round((skill.buffedBy[id] / skill.totalDamage) * 100),
                        syn.source.skill?.icon
                    );

                    buff.buffs.push(b);
                    synergyDamage += skill.buffedBy[id];
                }

                return;
            }

            if (skill.buffedBy[id]) {
                const b = new Buff(
                    syn.source.icon,
                    round((skill.buffedBy[id] / skill.totalDamage) * 100),
                    syn.source.skill?.icon
                );
                addBardBubbles(key, b, syn);
                buff.buffs.push(b);
                synergyDamage += skill.buffedBy[id];
            } else if (skill.debuffedBy[id]) {
                buff.buffs.push(
                    new Buff(
                        syn.source.icon,
                        round((skill.debuffedBy[id] / skill.totalDamage) * 100),
                        syn.source.skill?.icon
                    )
                );
                synergyDamage += skill.debuffedBy[id];
            }
        });

        if (synergyDamage > 0) {
            buff.percentage = round((synergyDamage / skill.totalDamage) * 100);
        }
        synergyPercentageDetails.push(buff);
    });

    return synergyPercentageDetails;
}

export function getSynergyPercentageDetailsSum(
    groupedSynergies: Map<string, Map<number, StatusEffect>>,
    skills: Skill[],
    damageStats: DamageStats
) {
    const totalDamage = damageStats.damageDealt;
    const totalDamageWithoutHa = totalDamage - (damageStats.hyperAwakeningDamage ?? 0);

    const synergyPercentageDetails: BuffDetails[] = [];
    groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        const buffs = new BuffDetails();
        buffs.id = key;
        let isHat = false;
        synergies.forEach((syn, id) => {
            isHat = supportSkills.haTechnique.includes(id);

            const buff = new Buff(syn.source.icon, "", syn.source.skill?.icon);
            addBardBubbles(key, buff, syn);
            let totalBuffed = 0;
            for (const skill of skills) {
                if (hyperAwakeningIds.has(skill.id) && !isHat) {
                    continue;
                }
                if (skill.buffedBy[id]) {
                    totalBuffed += skill.buffedBy[id];
                    synergyDamage += skill.buffedBy[id];
                } else if (skill.debuffedBy[id]) {
                    totalBuffed += skill.debuffedBy[id];
                    synergyDamage += skill.debuffedBy[id];
                }
            }
            if (isHat) {
                buff.percentage = round((totalBuffed / totalDamage) * 100);
            } else {
                buff.percentage = round((totalBuffed / totalDamageWithoutHa) * 100);
            }
            buffs.buffs.push(buff);
        });

        if (synergyDamage > 0) {
            if (isHat) {
                buffs.percentage = round((synergyDamage / totalDamage) * 100);
            } else {
                buffs.percentage = round((synergyDamage / totalDamageWithoutHa) * 100);
            }
        }
        synergyPercentageDetails.push(buffs);
    });

    return synergyPercentageDetails;
}

export function getPartyShields(
    players: Array<Entity>,
    encounterPartyInfo: PartyInfo,
    groupedShields: Map<string, Map<number, StatusEffect>>,
    tab: ShieldTab
) {
    const parties = new Array<Array<Entity>>();
    const partyPercentages = new Array<number[]>();
    const partyInfo = Object.entries(encounterPartyInfo);
    let shieldValue = "";
    let shieldBy = "";
    switch (tab) {
        case ShieldTab.GIVEN:
            shieldBy = "shieldsGivenBy";
            shieldValue = "shieldsGiven";
            break;
        case ShieldTab.RECEIVED:
            shieldBy = "shieldsReceivedBy";
            shieldValue = "shieldsReceived";
            break;
        case ShieldTab.E_GIVEN:
            shieldBy = "damageAbsorbedOnOthersBy";
            shieldValue = "damageAbsorbedOnOthers";
            break;
        case ShieldTab.E_RECEIVED:
            shieldBy = "damageAbsorbedBy";
            shieldValue = "damageAbsorbed";
            break;
    }
    const topShield = Math.max(...players.map((player) => player.damageStats[shieldValue]));
    const partyShields = new Map<string, Map<string, Array<ShieldDetails>>>();
    const partyGroupedShields = new Map<string, Set<string>>();

    if (partyInfo.length >= 1) {
        for (const [partyIdStr, names] of partyInfo) {
            const partyId = Number(partyIdStr);
            parties[partyId] = [];
            for (const name of names) {
                const player = players.find((player) => player.name === name);
                if (player) {
                    parties[partyId].push(player);
                }
            }
            if (parties[partyId] && parties[partyId].length > 0) {
                parties[partyId].sort((a, b) => b.damageStats[shieldValue] - a.damageStats[shieldValue]);
                partyPercentages[partyId] = parties[partyId].map(
                    (player) => (player.damageStats[shieldValue] / topShield) * 100
                );
            }
        }
    } else {
        parties[0] = players;
    }

    if (groupedShields.size > 0 && parties.length >= 1) {
        parties.forEach((party, partyId) => {
            partyGroupedShields.set(partyId.toString(), new Set<string>());
            const pShields = new Set<string>();
            for (const player of party) {
                groupedShields.forEach((shields, key) => {
                    shields.forEach((_, id) => {
                        if (player.damageStats[shieldBy][id]) {
                            pShields.add(key);
                        }
                    });
                });
            }
            partyGroupedShields.set(partyId.toString(), new Set([...pShields].sort()));
        });

        parties.forEach((party, partyId) => {
            partyShields.set(partyId.toString(), new Map<string, Array<ShieldDetails>>());
            for (const player of party) {
                partyShields.get(partyId.toString())!.set(player.name, []);
                const playerBuffs = partyShields.get(partyId.toString())!.get(player.name)!;
                partyGroupedShields.get(partyId.toString())?.forEach((key) => {
                    const shieldDetails = new ShieldDetails();
                    shieldDetails.id = key;
                    let shieldTotal = 0;
                    const buffs = groupedShields.get(key) || new Map();
                    buffs.forEach((syn, id) => {
                        if (player.damageStats[shieldBy][id]) {
                            const s = new Shield(id, syn.source.icon, player.damageStats[shieldBy][id]);
                            shieldDetails.buffs.push(s);
                            shieldTotal += player.damageStats[shieldBy][id];
                        }
                    });
                    shieldDetails.total = shieldTotal;
                    playerBuffs.push(shieldDetails);
                });
            }
        });
    }

    return { parties, partyPercentages, partyGroupedShields, partyShields };
}

export function getPartyBuffs(
    players: Array<Entity>,
    topDamageDealt: number,
    encounterPartyInfo: PartyInfo,
    groupedSynergies: Map<string, Map<number, StatusEffect>>
): PartyBuffs {
    const parties = new Array<Array<Entity>>();
    const partyGroupedSynergies = new Map<string, Set<string>>();
    const partyPercentages = new Array<number[]>();

    const partyBuffs = new Map<string, Map<string, Array<BuffDetails>>>();

    const partyInfo = Object.entries(encounterPartyInfo);
    if (partyInfo.length >= 2) {
        for (const [partyIdStr, names] of partyInfo) {
            const partyId = Number(partyIdStr);
            parties[partyId] = [];
            for (const name of names) {
                const player = players.find((player) => player.name === name);
                if (player) {
                    parties[partyId].push(player);
                }
            }
            if (parties[partyId] && parties[partyId].length > 0) {
                parties[partyId].sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                partyPercentages[partyId] = parties[partyId].map(
                    (player) => (player.damageStats.damageDealt / topDamageDealt) * 100
                );
            }
        }
    } else {
        parties[0] = players;
    }

    if (groupedSynergies.size > 0 && parties.length > 1) {
        parties.forEach((party, partyId) => {
            partyGroupedSynergies.set(partyId.toString(), new Set<string>());
            const partySyns = new Set<string>();
            for (const player of party) {
                groupedSynergies.forEach((synergies, key) => {
                    synergies.forEach((_, id) => {
                        if (player.damageStats.buffedBy[id] || player.damageStats.debuffedBy[id]) {
                            partySyns.add(key);
                        }
                    });
                });
            }
            partyGroupedSynergies.set(partyId.toString(), new Set([...partySyns].sort()));
        });

        parties.forEach((party, partyId) => {
            partyBuffs.set(partyId.toString(), new Map<string, Array<BuffDetails>>());
            for (const player of party) {
                partyBuffs.get(partyId.toString())!.set(player.name, []);
                const playerBuffs = partyBuffs.get(partyId.toString())!.get(player.name)!;

                partyGroupedSynergies.get(partyId.toString())?.forEach((key) => {
                    const buffDetails = new BuffDetails();
                    buffDetails.id = key;
                    let buffDamage = 0;
                    const buffs = groupedSynergies.get(key) || new Map();
                    const damageDealtWithHa = player.damageStats.damageDealt;
                    const damageDealtWithoutHa = damageDealtWithHa - (player.damageStats.hyperAwakeningDamage ?? 0);
                    let isHat = false;

                    buffs.forEach((syn, id) => {
                        let damageDealt = damageDealtWithHa;
                        if (supportSkills.haTechnique.includes(id)) {
                            isHat = true;
                            damageDealt = damageDealtWithoutHa;
                        }

                        if (player.damageStats.buffedBy[id]) {
                            const b = new Buff(
                                syn.source.icon,
                                round((player.damageStats.buffedBy[id] / damageDealt) * 100),
                                syn.source.skill?.icon
                            );
                            addBardBubbles(key, b, syn);
                            buffDetails.buffs.push(b);
                            buffDamage += player.damageStats.buffedBy[id];
                        } else if (player.damageStats.debuffedBy[id]) {
                            buffDetails.buffs.push(
                                new Buff(
                                    syn.source.icon,
                                    round((player.damageStats.debuffedBy[id] / damageDealt) * 100),
                                    syn.source.skill?.icon
                                )
                            );
                            buffDamage += player.damageStats.debuffedBy[id];
                        }
                    });
                    if (buffDamage > 0) {
                        buffDetails.percentage = round(
                            (buffDamage / (isHat ? damageDealtWithHa : damageDealtWithoutHa)) * 100
                        );
                    }

                    playerBuffs.push(buffDetails);
                });
            }
        });
    }

    return { parties, partyGroupedSynergies, partyPercentages, partyBuffs };
}

export function calculatePartyWidth(
    partyGroupedSynergies: Map<string, Set<string>>,
    remToPx: number,
    currentVw: number
) {
    const partyWidths: { [key: string]: string } = {};
    partyGroupedSynergies.forEach((synergies, partyId) => {
        const widthRem = synergies.size * 3.5 + 10;
        const widthPx = widthRem * remToPx;
        if (widthPx > currentVw - 2 * remToPx) {
            partyWidths[partyId] = `${widthRem}rem`;
        } else {
            partyWidths[partyId] = `calc(100vw - 4.5rem)`;
        }
    });

    return partyWidths;
}

export function addBardBubbles(key: string, buff: Buff, syn: StatusEffect) {
    if (key === "__bard_2_serenadeofcourage") {
        if (syn.source.desc.includes("15")) {
            buff.bonus = 15;
        } else if (syn.source.desc.includes("10")) {
            buff.bonus = 10;
        } else if (syn.source.desc.includes("5")) {
            buff.bonus = 5;
        }
    } else if (key === "_arcanist_190900") {
        // twisted fate
        if (syn.source.desc.includes("10")) {
            buff.bonus = 10;
        } else if (syn.source.desc.includes("20")) {
            buff.bonus = 20;
        } else if (syn.source.desc.includes("40")) {
            buff.bonus = 40;
        }
    }
}

const supportClasses = [classNameToClassId["Paladin"], classNameToClassId["Bard"], classNameToClassId["Artist"]];

function isSupportBuff(statusEffect: StatusEffect) {
    if (!statusEffect.source.skill) {
        return false;
    }

    return supportClasses.includes(statusEffect.source.skill.classId);
}

export const supportSkills = {
    marking: [
        21020, // Sound shock, Stigma, Harp of Rythm
        21290, // Sonatina
        31420, // Paint: Drawing Orchids
        36050, // Light Shock
        36080, // Sword of Justice
        36150, // God’s Decree (Godsent Law)
        36100 // Holy Explosion
    ],
    markingGrp: [
        210230, // Pala marking
        210230, // Artist marking
        210230 // Bard marking
    ],
    atkPwr: [
        21170, // Sonic Vibration
        21160, // Heavenly Tune
        31400, // Paint: Sunsketch
        31410, // Paint: Sun Well Skill
        36200, // Heavenly Blessings
        36170 // Wrath of God
    ],
    atkPwrGrp: [
        101105, // Pala atk power
        314004, // Artist atk power
        101204 // Bard atk power
    ],
    identity: [
        21140, // Serenade of Courage 1
        21141, // Serenade of Courage 2
        21142, // Serenade of Courage 3
        21143, // Serenade of Courage
        31050, // Moonfall 10%
        31051 // Moonfall 5%
        // 36800 // Holy Aura
    ],
    identityGrp: [
        368000, // Pala Holy aura group
        310501 // Artist Moonfal group
        // Bard Serenade of Courage - doesn't exist
    ],
    haTechnique: [
        362600, // Paladin
        212305, // Bard
        319503 // Artist
    ]
};

function makeSupportBuffKey(statusEffect: StatusEffect) {
    const skillId = statusEffect.source.skill?.id ?? 0;
    let key = "__";
    key += `${classesMap[statusEffect.source.skill?.classId ?? 0]}`;
    if (supportSkills.markingGrp.includes(statusEffect.uniqueGroup)) {
        key += "_1";
    } else if (supportSkills.atkPwrGrp.includes(statusEffect.uniqueGroup)) {
        key += "_0";
    } else if (
        supportSkills.identity.includes(skillId) ||
        supportSkills.identityGrp.includes(statusEffect.uniqueGroup)
    ) {
        key += "_2";
    } else if (supportSkills.haTechnique.includes(skillId)) {
        key += "_3";
    } else {
        key += "_4";
    }
    key += `_${statusEffect.uniqueGroup ? statusEffect.uniqueGroup : statusEffect.source.skill?.name}`;
    return key;
}

const buffCategories = {
    partySynergy: ["classskill", "identity", "ability", "arkpassive"],
    selfItemSynergy: ["pet", "cook", "battleitem", "dropsofether", "bracelet", "elixir"],
    setSynergy: ["set", "arkpassive"],
    selfSkillSynergy: ["classskill", "identity", "ability", "arkpassive"],
    other: ["etc", "arkpassive"]
};

function isPartySynergy(statusEffect: StatusEffect) {
    return (
        buffCategories.partySynergy.includes(statusEffect.buffCategory) &&
        statusEffect.target === StatusEffectTarget.PARTY
    );
}

function isSelfItemSynergy(statusEffect: StatusEffect) {
    return buffCategories.selfItemSynergy.includes(statusEffect.buffCategory);
}

function isSetSynergy(statusEffect: StatusEffect) {
    return buffCategories.setSynergy.includes(statusEffect.buffCategory);
}

function isSelfSkillSynergy(statusEffect: StatusEffect) {
    return buffCategories.selfSkillSynergy.includes(statusEffect.buffCategory);
}

function isOtherSynergy(statusEffect: StatusEffect) {
    return buffCategories.other.includes(statusEffect.buffCategory);
}

export function getSkillCastBuffs(
    hitDamage: number,
    buffs: number[],
    debuffs: number[],
    encounterDamageStats: EncounterDamageStats,
    supportBuffs: SkillChartSupportDamage,
    playerClassId: number = 0,
    buffType: string = "party",
    buffFilter: boolean = true
) {
    const groupedBuffs: Map<string, Array<StatusEffectWithId>> = new Map();

    for (const buffId of buffs) {
        if (encounterDamageStats.buffs.hasOwnProperty(buffId)) {
            includeBuff(
                hitDamage,
                buffId,
                encounterDamageStats.buffs[buffId],
                groupedBuffs,
                supportBuffs,
                playerClassId,
                buffType,
                buffFilter
            );
        }
    }
    for (const buffId of debuffs) {
        if (encounterDamageStats.debuffs.hasOwnProperty(buffId)) {
            includeBuff(
                hitDamage,
                buffId,
                encounterDamageStats.debuffs[buffId],
                groupedBuffs,
                supportBuffs,
                playerClassId,
                buffType,
                buffFilter
            );
        }
    }

    return new Map([...groupedBuffs].sort((a, b) => String(a[0]).localeCompare(b[0])));
}

export function getFormattedBuffString(groupedBuffs: Map<string, Array<StatusEffectWithId>>, iconPath: string) {
    let buffString = "";
    buffString += "<div class='flex'>";
    for (const [, buffs] of groupedBuffs) {
        for (const buff of buffs) {
            buffString += `<img class="size-6 rounded-sm" src="${iconPath + getSkillIcon(buff.statusEffect.source.icon)}" alt="${buff.statusEffect.source.skill?.name}"/>`;
        }
    }
    buffString += "</div>";
    return buffString;
}

function includeBuff(
    hitDamage: number,
    buffId: number,
    buff: StatusEffect,
    map: Map<string, Array<StatusEffectWithId>>,
    supportBuffs: SkillChartSupportDamage,
    playerClassId: number,
    buffType: string,
    buffFilter: boolean
) {
    let key = "";
    if (
        buffFilter &&
        ((StatusEffectBuffTypeFlags.DMG |
            StatusEffectBuffTypeFlags.CRIT |
            StatusEffectBuffTypeFlags.ATKSPEED |
            StatusEffectBuffTypeFlags.MOVESPEED |
            StatusEffectBuffTypeFlags.COOLDOWN) &
            buff.buffType) ===
            0
    ) {
        return;
    }
    if (buffType === "party" && isPartySynergy(buff)) {
        if (isSupportBuff(buff)) {
            key = makeSupportBuffKey(buff);
            if (key.includes("_0")) {
                supportBuffs.buff += hitDamage;
            } else if (key.includes("_1")) {
                supportBuffs.brand += hitDamage;
            } else if (key.includes("_2")) {
                supportBuffs.identity += hitDamage;
            }
        } else {
            key = `${classesMap[buff.source.skill?.classId ?? 0]}_${
                buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
            }`;
        }

        addToMap(key, buffId, buff, map);
    } else if (buffType === "self") {
        if (isPartySynergy(buff)) {
        } else if (isSelfSkillSynergy(buff)) {
            if (buff.buffCategory === "ability") {
                key = `${buff.uniqueGroup ? buff.uniqueGroup : buffId}`;
            } else {
                if (playerClassId !== buff.source.skill?.classId) {
                    return;
                }
                key = `_${classesMap[buff.source.skill?.classId ?? 0]}_${
                    buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
                }`;
            }
            addToMap(key, buffId, buff, map);
        } else if (isSetSynergy(buff)) {
            addToMap(`_set_${buff.source.setName}`, buffId, buff, map);
        }
    } else if (buffType === "misc") {
        if (isSelfItemSynergy(buff)) {
            if (buff.buffCategory === "bracelet") {
                // put bracelets buffs at the end
                key = `zzbracelet_${buff.uniqueGroup}`;
            } else if (buff.buffCategory === "elixir") {
                key = `elixir_${buff.uniqueGroup}`;
            } else {
                key = buff.buffCategory;
            }
            addToMap(key, buffId, buff, map);
        } else if (isOtherSynergy(buff)) {
            addToMap(`etc_${buff.source.name}`, buffId, buff, map);
        }
    }
}

function addToMap(key: string, buffId: number, buff: StatusEffect, map: Map<string, Array<StatusEffectWithId>>) {
    const buffWithId: StatusEffectWithId = { id: buffId, statusEffect: buff };
    if (map.has(key)) {
        map.get(key)?.push(buffWithId);
    } else {
        map.set(key, [buffWithId]);
    }
}

export const hyperAwakeningIds: Set<number> = new Set([
    16720, 16730, 18240, 18250, 17250, 17260, 36230, 36240, 45820, 45830, 19360, 19370, 20370, 20350, 21320, 21330,
    37380, 37390, 22360, 22370, 23400, 23410, 24300, 24310, 34620, 34630, 39340, 39350, 47300, 47310, 25410, 25420,
    27910, 27920, 26940, 26950, 46620, 46630, 29360, 29370, 30320, 30330, 35810, 35820, 38320, 38330, 31920, 31930,
    32290, 32280
]);
