import { settings } from "$lib/stores.svelte";
import { estherMap } from "$lib/constants/esthers";
import type { Entity } from "$lib/types";
import { round2 } from "./numbers";
import { missingInfo } from "./stores";

export function isValidName(word: string) {
  return /^\p{Lu}/u.test(word);
}

export function removeUnknownHtmlTags(input: string) {
  input = input.replace(/<\$TABLE_SKILLFEATURE[^>]*\/>/g, "??");
  input = input.replace(/<\$[^<>]*?(?:<[^<>]*?>[^<>]*?)*?\/?>/g, "??");
  return input;
}

export function formatPlayerName(player: Entity, generalSettings: any): string {
  let playerName = player.name;
  const validName = isValidName(playerName);
  if (!validName) {
    missingInfo.set(true);
  }
  if (!validName || !generalSettings.showNames) {
    if (player.class) {
      playerName = player.class;
    } else {
      playerName = "";
    }
  }
  if (generalSettings.hideNames) {
    playerName = "";
  }
  if (generalSettings.showGearScore && player.gearScore > 0) {
    playerName = round2(player.gearScore, 2) + " " + playerName;
  }
  if (player.isDead) {
    playerName = "💀 " + playerName;
  }

  return playerName;
}

export function getSkillIcon(skillIcon: string): string {
  return "/images/" + skillIcon !== "" ? skillIcon : "unknown.png";
}

export function getClassIcon(classIcon: number): string {
  return "/images/classes/" + classIcon + ".png";
}

export function getEstherFromNpcId(npcId: number): string {
  for (const esther of estherMap) {
    if (esther.npcs.includes(npcId)) return esther.name;
  }

  return "Unknown";
}