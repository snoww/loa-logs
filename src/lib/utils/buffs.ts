import { classesMap } from "$lib/constants/classes";
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
    type PartyInfo
} from "$lib/types";
import { round } from "./numbers";

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
    focusedPlayer: Entity | null,
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
    focusedPlayer: Entity | null,
    tab: MeterTab,
    buffFilter = true
) {
    let key = "";
    // Party synergies
    if (["classskill", "identity", "ability"].includes(buff.buffCategory) && buff.target === StatusEffectTarget.PARTY) {
        if (tab === MeterTab.PARTY_BUFFS) {
            if (buff.source.skill && [105, 204, 602].includes(buff.source.skill.classId)) {
                // put support buffs at the start when sorting
                key = "_";
            }
            key += `${classesMap[buff.source.skill?.classId ?? 0]}_${
                buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
            }`;
            groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
        }
    }
    // Self synergies
    else if (["pet", "cook", "battleitem", "dropsofether", "bracelet", "elixir"].includes(buff.buffCategory)) {
        if (tab === MeterTab.SELF_BUFFS) {
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
    } else if (["set"].includes(buff.buffCategory)) {
        if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
            // put set buffs at the start
            groupedSynergiesAdd(groupedSynergies, `_set_${buff.source.setName}`, id, buff, focusedPlayer, buffFilter);
        }
    } else if (["classskill", "identity", "ability"].includes(buff.buffCategory)) {
        // self & other identity, class skill, engravings
        if (tab === MeterTab.SELF_BUFFS && focusedPlayer) {
            if (buff.buffCategory === "ability") {
                key = `${buff.uniqueGroup ? buff.uniqueGroup : id}`;
            } else {
                if (focusedPlayer.classId !== buff.source.skill?.classId) return; // We hide other classes self buffs (class_skill & identity)
                key = `${classesMap[buff.source.skill?.classId ?? 0]}_${
                    buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
                }`;
            }
            groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
        }
    } else if (["etc"].includes(buff.buffCategory)) {
        if (tab === MeterTab.SELF_BUFFS && focusedPlayer) {
            groupedSynergiesAdd(groupedSynergies, `etc_${buff.source.name}`, id, buff, focusedPlayer, buffFilter);
        }
    }
}

export function getSynergyPercentageDetails(groupedSynergies: Map<string, Map<number, StatusEffect>>, skill: Skill) {
    const synergyPercentageDetails: BuffDetails[] = [];
    groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        const buff = new BuffDetails();
        buff.id = key;
        synergies.forEach((syn, id) => {
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
    totalDamage: number
) {
    const synergyPercentageDetails: BuffDetails[] = [];
    groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        const buffs = new BuffDetails();
        buffs.id = key;
        synergies.forEach((syn, id) => {
            const buff = new Buff(syn.source.icon, "", syn.source.skill?.icon);
            addBardBubbles(key, buff, syn);
            let totalBuffed = 0;
            for (const skill of skills) {
                if (skill.buffedBy[id]) {
                    totalBuffed += skill.buffedBy[id];
                    synergyDamage += skill.buffedBy[id];
                } else if (skill.debuffedBy[id]) {
                    totalBuffed += skill.debuffedBy[id];
                    synergyDamage += skill.debuffedBy[id];
                }
            }
            buff.percentage = round((totalBuffed / totalDamage) * 100);
            buffs.buffs.push(buff);
        });

        if (synergyDamage > 0) {
            buffs.percentage = round((synergyDamage / totalDamage) * 100);
        }
        synergyPercentageDetails.push(buffs);
    });

    return synergyPercentageDetails;
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
            const partySyns = partyGroupedSynergies.get(partyId.toString())!;
            for (const player of party) {
                groupedSynergies.forEach((synergies, key) => {
                    synergies.forEach((_, id) => {
                        if (player.damageStats.buffedBy[id] || player.damageStats.debuffedBy[id]) {
                            partySyns.add(key);
                        }
                    });
                });
            }
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
                    buffs.forEach((syn, id) => {
                        if (player.damageStats.buffedBy[id]) {
                            const b = new Buff(
                                syn.source.icon,
                                round((player.damageStats.buffedBy[id] / player.damageStats.damageDealt) * 100),
                                syn.source.skill?.icon
                            );
                            addBardBubbles(key, b, syn);
                            buffDetails.buffs.push(b);
                            buffDamage += player.damageStats.buffedBy[id];
                        } else if (player.damageStats.debuffedBy[id]) {
                            buffDetails.buffs.push(
                                new Buff(
                                    syn.source.icon,
                                    round((player.damageStats.debuffedBy[id] / player.damageStats.damageDealt) * 100),
                                    syn.source.skill?.icon
                                )
                            );
                            buffDamage += player.damageStats.debuffedBy[id];
                        }
                    });
                    if (buffDamage > 0) {
                        buffDetails.percentage = round((buffDamage / player.damageStats.damageDealt) * 100);
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
    if (key === "_bard_serenadeofcourage") {
        if (syn.source.desc.includes("15")) {
            buff.bonus = 15;
        } else if (syn.source.desc.includes("10")) {
            buff.bonus = 10;
        } else if (syn.source.desc.includes("5")) {
            buff.bonus = 5;
        }
    } else if (key === "arcanist_190900") {
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
