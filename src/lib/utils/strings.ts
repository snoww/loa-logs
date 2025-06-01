import { estherMap } from "$lib/constants/esthers";
import { settings } from "$lib/stores.svelte";
import type { Entity } from "$lib/types";
import { normalizeIlvl } from "./numbers";

/** Check if a name is valid */
export const isNameValid = (value: unknown): value is string =>
  typeof value === "string" && value.length >= 2 && !/\d/.test(value);

/** Remove all custom tags from lost ark data */
export function removeUnknownHtmlTags(input: string) {
  // capital sin of using regex to parse html
  input = input.replace(/<\$TABLE_SKILLFEATURE[^>]*\/>/g, "??");
  input = input.replace(/<\$[^<>]*?(?:<[^<>]*?>[^<>]*?)*?\/?>/g, "??");
  return input;
}

export function formatPlayerName(player: Entity): string {
  let playerName = player.name;
  const validName = isNameValid(playerName);
  // if (!validName) {
  //   missingInfo.set(true);
  // }
  if (!validName || !settings.app.general.showNames) {
    if (player.class) {
      playerName = player.class;
    } else {
      playerName = "";
    }
  }
  if (settings.app.general.hideNames) {
    playerName = "";
  }
  if (settings.app.general.showGearScore && player.gearScore > 0) {
    playerName = normalizeIlvl(player.gearScore) + " " + playerName;
  }
  if (player.isDead) {
    playerName = "💀 " + playerName;
  }

  return playerName;
}

export function getSkillIcon(skillIcon: string): string {
  return "/images/skills/" + (skillIcon !== "" ? skillIcon : "unknown.png");
}

export function getClassIcon(classIcon: number | string): string {
  return "/images/classes/" + classIcon + ".png";
}

export function getEstherFromNpcId(npcId: number): string {
  for (const esther of estherMap) {
    if (esther.npcs.includes(npcId)) return esther.name;
  }

  return "Unknown";
}

export function normalizeRegion(region: string) {
  if (region === "EUC") {
    return "CE";
  }

  return region;
}

// shade rgb
// from https://stackoverflow.com/a/13542669/11934162
export const rgbLinearShadeAdjust = (color: number | string, percentage: number = -0.2, alpha?: number): string => {
  if (typeof color === "string") {
    color = parseInt(color.replace("#", ""), 16);
  }
  if (!alpha) {
    alpha = color > 0xffffff ? (color >> 24) & 0xff : 1;
  }
  if (color > 0xffffff) {
    color >>= 8;
  }
  let a = (color >> 16) & 0xff;
  let b = (color >> 8) & 0xff;
  let c = color & 0xff;

  const r = Math.round;
  const lz = percentage < 0;
  const t = lz ? 0 : 255 * percentage;
  const P = lz ? 1 + percentage : 1 - percentage;
  return "rgba(" + r(a * P + t) + "," + r(b * P + t) + "," + r(c * P + t) + "," + alpha + ")";
};
