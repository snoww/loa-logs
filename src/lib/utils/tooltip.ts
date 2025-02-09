import {
    type ArkPassiveData,
    type ArkPassiveNode,
    type BuffDetails,
    type Entity,
    ShieldDetails,
    type Skill,
    type StatusEffect
} from "$lib/types";
import { createTippy } from "svelte-tippy";
import "tippy.js/animations/perspective-subtle.css";
import "tippy.js/dist/svg-arrow.css";
import { getSkillIcon, removeUnknownHtmlTags } from "./strings";
import { roundArrow } from "tippy.js";
import { classesMap } from "$lib/constants/classes";
import { abbreviateNumberSplit } from "$lib/utils/numbers";
import ArkPassives from "$lib/constants/ArkPassives.json";

export const tooltip = createTippy({
    allowHTML: true,
    arrow: roundArrow,
    placement: "bottom",
    animation: "perspective-subtle",
    theme: "buff"
});

export const menuTooltip = createTippy({
    allowHTML: true,
    arrow: roundArrow,
    placement: "bottom",
    animation: "perspective-subtle",
    theme: "menu"
});

export const skillTooltip = createTippy({
    allowHTML: true,
    arrow: roundArrow,
    placement: "bottom",
    animation: "perspective-subtle",
    theme: "buff"
});

export function generateTooltipContent(buffs: BuffDetails, iconPath: string) {
    let str = `<div class="font-normal text-xs flex flex-col space-y-1 -mx-px py-px">`;
    for (const buff of buffs.buffs) {
        if (buff.sourceIcon) {
            str += `<div class="flex items-center">`;
            str += `<img src=${
                iconPath + getSkillIcon(buff.sourceIcon)
            } alt="buff_source_icon" class="size-5 rounded mr-1"/>`;
            if (buff.bonus) {
                str += `[${buff.bonus}<span class="text-3xs text-gray-300">%</span>] `;
            }
            str += `${buff.percentage}<span class="text-3xs text-gray-300">%</span>`;
            str += `</div>`;
        } else {
            str += `<div class="flex items-center">`;
            str += `<img src=${iconPath + getSkillIcon(buff.icon)} alt="buff_icon" class="size-5 rounded mr-1"/>`;
            str += `${buff.percentage}<span class="text-3xs text-gray-300">%</span>`;
            str += `</div>`;
        }
    }
    str += "</div>";
    return str;
}

export function generateShieldTooltipContent(buffs: ShieldDetails, iconPath: string) {
    let str = `<div class="font-normal text-xs flex flex-col space-y-1 -mx-px py-px">`;
    for (const buff of buffs.buffs) {
        const shield = abbreviateNumberSplit(buff.value);
        str += `<div class="flex items-center">`;
        str += `<img src=${iconPath + getSkillIcon(buff.icon)} alt="buff_icon" class="size-5 rounded mr-1"/>`;
        str += `${shield[0]}<span class="text-3xs text-gray-300">${shield[1]}</span>`;
        str += `</div>`;
    }
    str += "</div>";
    return str;
}

export function generateHeaderTooltip(buff: StatusEffect, iconPath: string) {
    let str = `<div class="font-normal text-sm py-1">`;
    if (buff.source.skill) {
        str += `<div class="flex">`;
        str += `${classesMap[buff.source.skill.classId]}:`;
        str += `<img src=${iconPath + getSkillIcon(buff.source.skill.icon)} alt=${
            buff.source.skill.name
        } class="size-5 mx-1"/>`;
        str += buff.source.skill.name;
        str += `</div>`;
    } else {
        str += `<div class="flex">`;
        if (buff.buffCategory === "set" && buff.source.setName) {
            str += `<div class="pr-1">`;
            str += "[Set] " + buff.source.setName + ":";
            str += `</div>`;
        } else if (buff.buffCategory === "bracelet") {
            str += `<div class="pr-1">`;
            str += "[Bracelet]";
            str += `</div>`;
        } else if (buff.buffCategory === "elixir") {
            str += `<div class="pr-1">`;
            str += "[Elixir]";
            str += `</div>`;
        } else if (buff.buffCategory === "battleitem") {
            str += `<div class="pr-1">`;
            str += "[Battle Item]";
            str += `</div>`;
        } else if (buff.buffCategory === "dropsofether") {
            str += `<div class="pr-1">`;
            str += "[Drops of Ether]";
            str += `</div>`;
        }
        str += removeUnknownHtmlTags(buff.source.name);
        str += `</div>`;
    }
    str += `<div class="flex tracking-tight items-center">`;
    str += `<img src=${iconPath + getSkillIcon(buff.source.icon)} alt=${buff.name} class="size-5 mr-1"/>`;
    str += `<div class="">`;
    str += removeUnknownHtmlTags(buff.source.desc);
    str += `</div></div></div>`;

    return str;
}

export function generateClassTooltip(player: Entity) {
    let str = `<div class="flex">`;
    if (player.arkPassiveActive) {
        str += `<div class="mr-1"><span class="text-purple-400">[Ark Passive]</span></div>`;
    }
    if (player.spec) {
        str += player.spec + " " + player.class;
    } else {
        str += player.class;
    }

    str += "</div>";
    return str;
}

export function generateSkillTooltip(skill: Skill) {
    let str = `<div class="py-0.5">${skill.name}</div>`;
    str += "<div class='text-gray-300'>";
    if (skill.gemDamage) {
        str += `<div><span style="color: ${getColorFromTier(skill.gemTierDmg ?? skill.gemTier)}">T${skill.gemTierDmg ?? skill.gemTier ?? 3} </span><span style="color: ${getColorFromLevel(skill.gemDamage, skill.gemTierDmg ?? skill.gemTier)}">Lv. ${skill.gemDamage}</span> DMG</div>`;
    }
    if (skill.gemCooldown) {
        str += `<div><span style="color: ${getColorFromTier(skill.gemTier)}">T${skill.gemTier ?? 3} </span><span style="color: ${getColorFromLevel(skill.gemCooldown, skill.gemTier)}">Lv. ${skill.gemCooldown}</span> CD</div>`;
    }
    str += "</div>";
    if (!skill.tripodIndex) {
        return str;
    }
    str += `<div class="w-16">`;
    if (skill.tripodIndex.first > 0) {
        str += `<div class="flex space-x-1 py-0.5 justify-center">`;
        if (skill.tripodIndex.first === 1) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-blue-800"><p class="text-gray-200">${skill.tripodLevel?.first || 1}</p></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
        } else if (skill.tripodIndex.first === 2) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-blue-800"><p class="text-gray-200">${skill.tripodLevel?.first || 1}</p></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
        } else if (skill.tripodIndex.first === 3) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-blue-800"><p class="text-gray-200">${skill.tripodLevel?.first || 1}</p></div>`;
        }
        str += `</div>`;
    }
    if (skill.tripodIndex.second > 0) {
        str += `<div class="flex space-x-1 py-0.5 justify-center">`;
        if (skill.tripodIndex.second === 1) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-lime-600"><p class="text-gray-200">${skill.tripodLevel?.second || 1}</p></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
        } else if (skill.tripodIndex.second === 2) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-lime-600"><p class="text-gray-200">${skill.tripodLevel?.second || 1}</p></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
        } else if (skill.tripodIndex.second === 3) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-lime-600"><p class="text-gray-200">${skill.tripodLevel?.second || 1}</p></div>`;
        }
        str += `</div>`;
    }
    if (skill.tripodIndex.third > 0) {
        str += `<div class="flex space-x-1 py-0.5 justify-center">`;
        if (skill.tripodIndex.third === 1) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-amber-600"><p class="text-gray-200">${skill.tripodLevel?.third || 1}</p></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
        } else if (skill.tripodIndex.third === 2) {
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-gray-800"></div>`;
            str += `<div class="flex size-5 items-center justify-center rounded-full bg-amber-600"><p class="text-gray-200">${skill.tripodLevel?.third || 1}</p></div>`;
        }
        str += `</div>`;
    }
    str += `</div>`;

    return str;
}

export function generateArkPassiveTooltip(name: string, arkPassiveData?: ArkPassiveData, spec?: string) {
    let str = `<div>${name}</div>`;
    if (arkPassiveData && spec) {
        let apSpec = "";
        if (arkPassiveData.enlightenment) {
            for (const node of arkPassiveData.enlightenment) {
                const specName = getSpecFromArkPassive(node);
                if (specName !== "Unknown") {
                    apSpec = specName;
                    break;
                }
            }
        }

        if (apSpec === spec) {
            if (arkPassiveData.evolution) {
                str += `<div class="text-purple-400">[Evolution]</div>`;
                str += `<div class="flex flex-col">`;
                for (const node of arkPassiveData.evolution) {
                    // @ts-expect-error no type mapping
                    const apNode = ArkPassives[node.id.toString()];
                    const apName: string = apNode.levels[node.lv.toString()].name;
                    str += `<div class="text-orange-200">${apName} <span class="text-white">Lv. ${node.lv}</span></div>`;
                }
                str += `</div>`;
            }
            if (arkPassiveData.enlightenment) {
                str += `<div class="text-purple-400">[Enlightenment]</div>`;
                str += `<div class="flex flex-col">`;
                for (const node of arkPassiveData.enlightenment) {
                    // @ts-expect-error no type mapping
                    const apNode = ArkPassives[node.id.toString()];
                    const apName: string = apNode.levels[node.lv.toString()].name;
                    str += `<div class="text-cyan-200">${apName} <span class="text-white">Lv. ${node.lv}</span></div>`;
                }
                str += `</div>`;
            }
            if (arkPassiveData.leap) {
                str += `<div class="text-purple-400">[Leap]</div>`;
                str += `<div class="flex flex-col">`;
                for (const node of arkPassiveData.leap) {
                    // @ts-expect-error no type mapping
                    const apNode = ArkPassives[node.id.toString()];
                    const apName: string = apNode.levels[node.lv.toString()].name;
                    str += `<div class="text-lime-400">${apName} <span class="text-white">Lv. ${node.lv}</span></div>`;
                }
                str += `</div>`;
            }
        } else {
            str += `<div class="text-violet-400">Mismatched Ark Passive Data</div>`;
        }
    }

    return str;
}

function getColorFromLevel(level: number, tier?: number) {
    if (tier === 4) {
        if (level === 3 || level === 4) {
            return "#a23ac7";
        } else if (level === 5 || level === 6 || level === 7) {
            return "#e08d14";
        } else if (level === 8 || level === 9) {
            return "#ed691a";
        } else if (level === 10) {
            return "#e7c990";
        } else {
            return "#e5e7eb";
        }
    } else {
        if (level === 5 || level === 6) {
            return "#a23ac7";
        } else if (level === 7 || level === 8 || level === 9) {
            return "#e08d14";
        } else if (level === 10) {
            return "#ed691a";
        } else {
            return "#e5e7eb";
        }
    }
}

function getColorFromTier(tier?: number) {
    if (tier === 4) {
        return "#d96dff";
    } else {
        return "#e08d14";
    }
}

function getSpecFromArkPassive(node: ArkPassiveNode): string {
    switch (node.id) {
        case 2160000:
            return "Berserker Technique";
        case 2160010:
            return "Mayhem";
        case 2170000:
            return "Lone Knight";
        case 2170010:
            return "Combat Readiness";
        case 2180000:
            return "Rage Hammer";
        case 2180010:
            return "Gravity Training";
        case 2360000:
            return "Judgement";
        case 2360010:
            return "Blessed Aura";
        case 2450000:
            return "Punisher";
        case 2450010:
            return "Predator";
        case 2230000:
            return "Ultimate Skill: Taijutsu";
        case 2230100:
            return "Shock Training";
        case 2220000:
            return "First Intention";
        case 2220100:
            return "Esoteric Skill Enhancement";
        case 2240000:
            return "Energy Overflow";
        case 2240100:
            return "Robust Spirit";
        case 2340000:
            return "Control";
        case 2340100:
            return "Pinnacle";
        case 2470000:
            return "Brawl King Storm";
        case 2470100:
            return "Asura's Path";
        case 2390000:
            return "Esoteric Flurry";
        case 2390010:
            return "Deathblow";
        case 2300000:
            return "Barrage Enhancement";
        case 2300100:
            return "Firepower Enhancement";
        case 2290000:
            return "Enhanced Weapon";
        case 2290100:
            return "Pistoleer";
        case 2280000:
            return "Death Strike";
        case 2280100:
            return "Loyal Companion";
        case 2350000:
            return "Evolutionary Legacy";
        case 2350100:
            return "Arthetinean Skill";
        case 2380000:
            return "Peacemaker";
        case 2380100:
            return "Time to Hunt";
        case 2370000:
            return "Igniter";
        case 2370100:
            return "Reflux";
        case 2190000:
            return "Grace of the Empress";
        case 2190100:
            return "Order of the Emperor";
        case 2200000:
            return "Communication Overflow";
        case 2200100:
            return "Master Summoner";
        case 2210000:
            return "Desperate Salvation";
        case 2210100:
            return "True Courage";
        case 2270000:
            return "Demonic Impulse";
        case 2270600:
            return "Perfect Suppression";
        case 2250000:
            return "Surge";
        case 2250600:
            return "Remaining Energy";
        case 2260000:
            return "Lunar Voice";
        case 2260600:
            return "Hunger";
        case 2460000:
            return "Full Moon Harvester";
        case 2460600:
            return "Night's Edge";
        case 2320000:
            return "Wind Fury";
        case 2320600:
            return "Drizzle";
        case 2310000:
            return "Full Bloom";
        case 2310600:
            return "Recurrence";
        default:
            return "Unknown";
    }
}
