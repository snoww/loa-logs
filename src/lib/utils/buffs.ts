import { classesMap } from "$lib/constants/classes";
import {
    StatusEffectBuffTypeFlags,
    type StatusEffect,
    type Entity,
    StatusEffectTarget,
    MeterTab,
    Buff,
    BuffDetails,
    type Skill
} from "$lib/types";
import { round } from "./numbers";

export function defaultBuffFilter(buffType: number): boolean {
    return (
        ((StatusEffectBuffTypeFlags.DMG |
            StatusEffectBuffTypeFlags.CRIT |
            StatusEffectBuffTypeFlags.ATKSPEED |
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
    // Party synergies
    if (["classskill", "identity", "ability"].includes(buff.buffCategory) && buff.target === StatusEffectTarget.PARTY) {
        if (tab === MeterTab.PARTY_BUFFS) {
            const key = `${classesMap[buff.source.skill?.classId ?? 0]}_${
                buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name
            }`;
            groupedSynergiesAdd(groupedSynergies, key, id, buff, focusedPlayer, buffFilter);
        }
    }
    // Self synergies
    else if (["pet", "cook", "battleitem", "dropsofether", "bracelet"].includes(buff.buffCategory)) {
        if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
            groupedSynergiesAdd(groupedSynergies, buff.buffCategory, id, buff, focusedPlayer, buffFilter);
        }
    } else if (["set"].includes(buff.buffCategory)) {
        if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
            groupedSynergiesAdd(groupedSynergies, `set_${buff.source.setName}`, id, buff, focusedPlayer, buffFilter);
        }
    } else if (["classskill", "identity", "ability"].includes(buff.buffCategory)) {
        // self & other identity, classskill, engravings
        if (tab === MeterTab.SELF_BUFFS && focusedPlayer) {
            let key;
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
                buff.buffs.push(
                    new Buff(
                        syn.source.icon,
                        round((skill.buffedBy[id] / skill.totalDamage) * 100),
                        syn.source.skill?.icon
                    )
                );
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

export function getSynergyPercentageDetailsSum(groupedSynergies: Map<string, Map<number, StatusEffect>>, skills: Skill[], totalDamage: number) {
    const synergyPercentageDetails: BuffDetails[] = [];
    groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        const buffs = new BuffDetails();
        buffs.id = key;
        synergies.forEach((syn, id) => {
            const buff = new Buff(
                syn.source.icon,
                "",
                syn.source.skill?.icon
            );
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