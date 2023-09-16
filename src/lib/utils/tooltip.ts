import type { BuffDetails, StatusEffect } from "$lib/types";
import { createTippy } from "svelte-tippy";
import "tippy.js/animations/perspective-subtle.css";
import "tippy.js/dist/svg-arrow.css";
import { getSkillIcon, removeUnknownHtmlTags } from "./strings";
import { roundArrow } from "tippy.js";
import { classesMap } from "$lib/constants/classes";

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
            } alt="buff_source_icon" class="w-5 h-5 rounded mr-1"/>`;
            str += `${buff.percentage}<span class="text-3xs text-gray-300">%</span>`;
            str += `</div>`;
        } else {
            str += `<div class="flex items-center">`;
            str += `<img src=${iconPath + getSkillIcon(buff.icon)} alt="buff_icon" class="w-5 h-5 rounded mr-1"/>`;
            str += `${buff.percentage}<span class="text-3xs text-gray-300">%</span>`;
            str += `</div>`;
        }
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
        } class="w-5 h-5 mx-1"/>`;
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
    str += `<img src=${iconPath + getSkillIcon(buff.source.icon)} alt=${buff.name} class="w-5 h-5 mr-1"/>`;
    str += `<div class="">`;
    str += removeUnknownHtmlTags(buff.source.desc);
    str += `</div></div></div>`;

    return str;
}
