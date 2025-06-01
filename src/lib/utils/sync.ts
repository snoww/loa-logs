import { addToast } from "$lib/components/Toaster.svelte";
import { settings } from "$lib/stores.svelte";
import type { Encounter } from "$lib/types";
import { invoke } from "@tauri-apps/api";
import pako from "pako";
import { uploadError, uploadTokenError } from "./toasts";

export const LOG_SITE_URL = "https://logs.snow.xyz";
export const API_URL = "https://api.snow.xyz";
// export const API_URL = "http://localhost:5180";

export const supportedBosses = [
  "Dark Mountain Predator",
  "Ravaged Tyrant of Beasts",
  // "Incubus Morphe",
  // "Nightmarish Morphe",
  "Covetous Devourer Vykas",
  "Covetous Legion Commander Vykas",
  "Saydon",
  "Kakul",
  "Encore-Desiring Kakul-Saydon",
  "Brelshaza, Monarch of Nightmares",
  "Phantom Legion Commander Brelshaza",
  // "Kaltaya, the Blooming Chaos",
  // "Rakathus, the Lurking Arrogance",
  // "Firehorn, Trampler of Earth",
  // "Lazaram, the Trailblazer",
  // "Gargadeth",
  // "Hanumatan",
  "Caliligos",
  "Achates",
  // "Veskal",
  // "Argeos",
  // "Killineza the Dark Worshipper",
  // "Valinak, Herald of the End",
  // "Thaemine the Lightqueller",
  // "Thaemine, Conqueror of Stars",
  // "Red Doom Narkiel",
  // "Covetous Master Echidna",
  "Behemoth, the Storm Commander",
  "Behemoth, Cruel Storm Slayer",
  "Akkan, Lord of Death",
  "Aegir, the Oppressor",
  "Narok the Butcher",
  "Phantom Manifester Brelshaza"
];

export async function uploadLog(id: number | string, encounter: Encounter) {
  if (
    !supportedBosses.includes(encounter.currentBossName) ||
    !encounter.cleared ||
    !encounter.bossOnlyDamage ||
    !encounter.difficulty
  ) {
    return;
  }

  const jsonString = JSON.stringify(encounter);
  const compressedData = pako.gzip(jsonString);
  const blob = new Blob([compressedData], { type: "application/octet-stream" });

  const resp = await fetch(API_URL + "/logs/upload", {
    method: "POST",
    headers: {
      access_token: settings.sync.accessToken,
      "Content-Encoding": "gzip",
      "Content-Type": "application/json",
      visibility: settings.sync.visibility ?? ""
    },
    body: blob
  });

  // basic errors
  if (!resp.ok && (resp.status === 500 || resp.status === 401)) {
    let error = "";
    if (resp.status == 500) {
      error = "server bwonk";
      addToast(uploadError("server bwonk"));
    } else if (resp.status == 401) {
      error = "invalid access token";
      addToast(uploadTokenError);
    }

    await invoke("write_log", {
      message: `couldn't upload encounter ${id} (${encounter.currentBossName}) - error: ${error}`
    });
    return;
  }

  // handle response
  const body = await resp.json();
  // failed server side encounter validation
  if (body.error) {
    if (body.error === "duplicate log" && body.duplicate) {
      const duplicate = body.duplicate;
      await invoke("write_log", {
        message: `did not upload duplicate encounter ${id} (${encounter.currentBossName}) using existing upstream: ${duplicate}`
      });
      await invoke("sync", { encounter: Number(id), upstream: duplicate.toString(), failed: false });
      return duplicate;
    }

    await invoke("write_log", {
      message: `couldn't upload encounter ${id} (${encounter.currentBossName}) - error: ${body.error.toLowerCase()}`
    });
    addToast(uploadError(body.error));
    await invoke("sync", { encounter: Number(id), upstream: "0", failed: true });
    return;
  }

  // successful upload
  const upstream = body.id;

  await invoke("write_log", {
    message: `uploaded encounter ${id} (${encounter.currentBossName}) upstream: ${upstream}`
  });
  await invoke("sync", { encounter: Number(id), upstream: upstream.toString(), failed: false });
  return upstream;
}

export async function checkAccessToken(accessToken: string) {
  if (!accessToken) {
    return false;
  }

  const resp = await fetch(API_URL + "/logs/token", {
    method: "GET",
    headers: {
      access_token: accessToken
    }
  });

  return resp.ok;
}
