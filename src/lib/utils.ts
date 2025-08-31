import { bossHpMap } from "$lib/constants/encounters";
import { estherMap } from "$lib/constants/esthers";
import { BossHpLog, type DamageStats, type Entity, type IdentityLogType, type IdentityLogTypeValue } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { writeImage } from "@tauri-apps/plugin-clipboard-manager";
import { check as checkUpdate } from "@tauri-apps/plugin-updater";
import html2canvas from "html2canvas-pro";
import { addToast } from "./components/Toaster.svelte";
import { screenshot, settings, updateInfo } from "./stores.svelte";
import { screenshotError, screenshotSuccess } from "./utils/toasts";

export const UWUOWO_URL = "https://uwuowo.mathi.moe";

export async function takeScreenshot(div?: HTMLElement) {
  if (!div) {
    return;
  }
  screenshot.take();
  setTimeout(async () => {
    const canvas = await html2canvas(div, { useCORS: true, backgroundColor: "#27272A" });
    canvas.toBlob(async (blob) => {
      if (!blob) return;
      try {
        await writeImage(await blob.arrayBuffer());
        addToast(screenshotSuccess);
      } catch (error) {
        addToast(screenshotError);
        invoke("write_log", { message: "failed to take screenshot: " + error });
      } finally {
        screenshot.done();
      }
    });
  }, 100);
}

export async function checkForUpdate() {
  try {
    const manifest = await checkUpdate();

    if (manifest !== null) {
      updateInfo.available = true;
      updateInfo.manifest = manifest;
    } else {
      updateInfo.available = false;
    }

    return updateInfo.available;
  } catch (e) {
    await invoke("write_log", { message: e });
  }
}

export function tryParseInt(intString: string | number, defaultValue = 0) {
  if (typeof intString === "number") {
    if (isNaN(intString)) return defaultValue;
    return intString;
  }

  let intNum;

  try {
    intNum = parseInt(intString);
    if (isNaN(intNum)) intNum = defaultValue;
  } catch {
    intNum = defaultValue;
  }

  return intNum;
}

/** Normalize an ilvl */
export const normalizeIlvl = (ilvl: number) => ilvl.toFixed(2).replace(/\.?0+$/, "");

/**
 * Round a number to a specific number of decimal places if necessary, returns a string
 */
export function customRound(num: number, decimalPlaces = 1) {
  const p = Math.pow(10, decimalPlaces);
  const n = num * p * (1 + Number.EPSILON);
  return (Math.round(n) / p).toString();
}

export function abbreviateNumber(n: number, round = 1) {
  if (n >= 1e3 && n < 1e6) return (n / 1e3).toFixed(1) + "k";
  if (n >= 1e6 && n < 1e9) return +(n / 1e6).toFixed(1) + "m";
  if (n >= 1e9 && n < 1e12) return +(n / 1e9).toFixed(round) + "b";
  if (n >= 1e12) return +(n / 1e12).toFixed(round) + "t";
  else return tryParseInt(n).toFixed(0);
}

/** Abbreviates a number into more compact representation, returning an array with the truncated number and the abbreviation */
export function abbreviateNumberSplit(n: number): [number, string] {
  if (n >= 1e3 && n < 1e6) return [+(n / 1e3).toFixed(1), "k"];
  if (n >= 1e6 && n < 1e9) return [+(n / 1e6).toFixed(1), "m"];
  if (n >= 1e9 && n < 1e12) return [+(n / 1e9).toFixed(1), "b"];
  if (n >= 1e12) return [+(n / 1e12).toFixed(1), "t"];
  else return [+n.toFixed(0), ""];
}

/**
 * Returns time from timestamp in minutes and seconds - 00:00
 *
 * when useText is true - 0m00s
 */
export function timestampToMinutesAndSeconds(
  millis: number,
  useText: boolean = false,
  showMs = false,
  extraPad = false
): string {
  const hoursmillis = millis % (60 * 60 * 1000);
  const minutes = Math.floor(hoursmillis / (60 * 1000));
  const minutesmillis = millis % (60 * 1000);
  const sec = Math.floor(minutesmillis / 1000);

  return useText
    ? String(minutes).padStart(extraPad ? 2 : 1, "0") +
        "m" +
        String(sec).padStart(2, "0") +
        "s" +
        (showMs ? String(millis % 1000).padStart(3, "0") + "ms" : "")
    : String(minutes).padStart(extraPad ? 2 : 1, "0") +
        ":" +
        String(sec).padStart(2, "0") +
        (showMs ? "." + String(millis % 1000).padStart(3, "0") : "");
}

export function formatTimestamp(timestampMs: number): string {
  const timestampDate = new Date(timestampMs);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const dateFormat: Intl.DateTimeFormatOptions = {
    hour: "numeric",
    minute: "2-digit"
  };
  let formattedDate = timestampDate.toLocaleString(undefined, dateFormat);
  if (timestampDate.toDateString() === today.toDateString()) {
    formattedDate = `Today @ ${formattedDate}`;
  } else if (timestampDate.toDateString() === yesterday.toDateString()) {
    formattedDate = `Yesterday @ ${formattedDate}`;
  } else {
    formattedDate = timestampDate
      .toLocaleString(undefined, {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "numeric",
        minute: "2-digit"
      })
      .replace(",", " ");
  }
  return formattedDate;
}

export function formatTimestampDate(timestampMs: number, iso = false): string {
  if (iso) {
    return new Date(timestampMs).toLocaleDateString("sv");
  }
  return new Date(timestampMs).toLocaleString(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit"
  });
}

export function formatTimestampTime(timestampMs: number): string {
  return new Date(timestampMs).toLocaleString(undefined, {
    hour: "numeric",
    minute: "2-digit"
  });
}

export function fillMissingElapsedTimes(data: IdentityLogType): IdentityLogType {
  const filledData: IdentityLogType = [];
  let lastValue: IdentityLogTypeValue;

  data.forEach((item, index) => {
    const [elapsedTime, value] = item;

    if (index > 0) {
      const [prevElapsedTime] = data[index - 1];

      for (let i = prevElapsedTime + 1; i < elapsedTime; i++) {
        filledData.push([i, lastValue]);
      }
    }

    filledData.push(item);
    lastValue = value;
  });

  return filledData;
}

export function formatMinutes(minutesDecimal: number): string {
  // Convert minutes to seconds
  const totalSeconds = Math.round(minutesDecimal * 60);

  // Calculate the number of whole minutes and the remaining seconds
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  // Format the result as a readable string
  let result = "";
  if (minutes > 0) {
    result = `${minutes}m`;
  }
  result += `${seconds}s`;

  return result;
}

export function resampleData(data: Array<BossHpLog>, interval = 5, length: number) {
  const resampledData: Array<BossHpLog> = [];
  let last = null;
  const lastTime = data[data.length - 1].time;

  const dataMap = data.reduce((map, obj) => {
    map.set(obj.time, obj);
    return map;
  }, new Map<number, BossHpLog>());

  for (let i = 0; i < length; i++) {
    const time = i * interval;
    if (time > lastTime) {
      break;
    }
    if (dataMap.has(time)) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      last = dataMap.get(time)!;
    } else if (last === null) {
      continue;
    }
    resampledData.push(new BossHpLog(time, last.hp, last.p));
  }

  return resampledData;
}

export function timeToSeconds(time: string): number {
  const split = time.split(":");
  const minutes = +split[0];
  const seconds = +split[1];
  return minutes * 60 + seconds;
}

export function getRDamage(damageStats: DamageStats): number {
  return (
    damageStats.damageDealt -
    damageStats.rdpsDamageReceivedSupport -
    (damageStats.rdpsDamageReceived - damageStats.rdpsDamageReceivedSupport) +
    damageStats.rdpsDamageGiven
  );
}

export function getBaseDamage(damageStats: DamageStats): number {
  return (
    damageStats.damageDealt -
    damageStats.rdpsDamageReceivedSupport -
    (damageStats.rdpsDamageReceived - damageStats.rdpsDamageReceivedSupport)
  );
}

export function getBossHpBars(bossName: string, bossMaxHp: number) {
  if (bossName === "Phantom Legion Commander Brelshaza") {
    if (bossMaxHp > 100_000_000_000) {
      return 420;
    } else {
      return 250;
    }
  } else {
    return bossHpMap[bossName];
  }
} // shade rgb
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

export function getClassIcon(classId: number | string): string {
  return "/images/classes/" + classId + ".png";
}

export function getEstherFromNpcId(npcId: number): string {
  for (const esther of estherMap) {
    if (esther.npcs.includes(npcId)) return esther.name;
  }

  return "Unknown";
}

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

export const normalize = (val: number, min: number, max: number): number => {
  return (val - min) / (max - min);
};

export const isSupportSpec = (spec?: string): boolean =>
  spec === "Desperate Salvation" || spec === "Full Bloom" || spec === "Blessed Aura" || spec === "Liberator";
