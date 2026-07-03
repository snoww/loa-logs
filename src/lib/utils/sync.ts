import { fetch } from "@tauri-apps/plugin-http";
import { addToast } from "$lib/components/Toaster.svelte";
import { settings } from "$lib/stores.svelte";
import { type Encounter, EntityType } from "$lib/types";
import pako from "pako";
import { uploadError, uploadTokenError } from "./toasts";
import { sync, writeLog } from "$lib/api";

export const API_URL = "https://api.snow.xyz";
// export const API_URL = "http://localhost:5180";

type SupportedBosses = Record<string, string[]>;

let supportedBossesCache: SupportedBosses | undefined;
let supportedBossesFetch: Promise<SupportedBosses> | undefined;

async function getSupportedBosses(forceRefresh = false) {
  if (forceRefresh) {
    supportedBossesCache = undefined;
    supportedBossesFetch = undefined;
  }

  if (supportedBossesCache) {
    return supportedBossesCache;
  }

  supportedBossesFetch ??= (async () => {
    const resp = await fetch(API_URL + "/logs/upload/bosses");
    if (!resp.ok) {
      throw new Error(`could not fetch supported bosses list ${resp.status}`);
    }

    const body: SupportedBosses = await resp.json();
    supportedBossesCache = body;
    return body;
  })().finally(() => {
    supportedBossesFetch = undefined;
  });

  return supportedBossesFetch;
}

function supportedBosses(encounter: Encounter, bosses: SupportedBosses) {
  if (!encounter.difficulty) {
    return false;
  }

  const difficulties = bosses[encounter.currentBossName];
  return !!difficulties && (difficulties.length === 0 || difficulties.includes(encounter.difficulty));
}

export async function uploadLog(
  id: number | string,
  encounter: Encounter,
  showToast = true,
  bulk = false,
  refreshSupportedBosses = false
) {
  if (!encounter.bossOnlyDamage) {
    if (showToast && !bulk) {
      addToast(uploadError("Boss only damage not enabled for this log", id));
    }

    return;
  }

  if (
    !encounter.cleared ||
    !encounter.difficulty ||
    encounter.difficulty === "Solo" ||
    Object.values(encounter.entities).filter((e) => e.entityType === EntityType.PLAYER).length > 8
  ) {
    if (showToast && !bulk) {
      addToast(uploadError("Log not supported for upload", id));
    }

    return;
  }

  try {
    const supportedBossesList = await getSupportedBosses(refreshSupportedBosses);
    if (!supportedBosses(encounter, supportedBossesList)) {
      if (showToast && !bulk) {
        addToast(uploadError("Boss not supported for upload", id));
      }

      return;
    }
  } catch {
    await writeLog("failed to check upload support");
  }

  const jsonString = JSON.stringify(encounter);
  const compressedData = pako.gzip(jsonString);
  const blob = new Blob([compressedData], { type: "application/octet-stream" });

  const resp = await fetch(API_URL + "/logs/upload", {
    method: "POST",
    headers: {
      access_token: settings.sync.accessToken,
      "Content-Encoding": "gzip",
      "Content-Type": "application/json"
    },
    body: blob
  });

  // basic errors
  if (!resp.ok && resp.status !== 400) {
    let error = "";
    if (resp.status == 401) {
      error = "invalid access token";
      if (showToast) addToast(uploadTokenError);
    } else {
      error = await resp.text();
      if (showToast) addToast(uploadError("server bwonk", id));
    }

    await writeLog(`couldn't upload encounter ${id} (${encounter.currentBossName}) - error: ${error}`);
    return;
  }

  // handle response
  const body = await resp.json();
  // failed server side encounter validation
  if (body.error) {
    if (body.duplicate) {
      const duplicate = body.duplicate;
      await writeLog(
        `did not upload duplicate encounter ${id} (${encounter.currentBossName}) using existing upstream: ${duplicate}`
      );
      await sync({
        encounter: Number(id),
        upstream: duplicate.toString(),
        failed: false
      });
      return duplicate;
    }

    await writeLog(
      `couldn't upload encounter ${id} (${encounter.currentBossName}) - error: ${body.error.toLowerCase()}`
    );
    if ((showToast && !bulk) || (bulk && !body.error.includes("Boss not supported")))
      addToast(uploadError(body.error, id));
    await sync({ encounter: Number(id), upstream: "0", failed: true });
    return;
  }

  // successful upload
  const upstream = body.id;

  await writeLog(`uploaded encounter ${id} (${encounter.currentBossName}) upstream: ${upstream}`);
  await sync({
    encounter: Number(id),
    upstream: upstream.toString(),
    failed: false
  });
  return upstream;
}

export async function checkAccessToken(accessToken: string) {
  if (!accessToken) {
    return false;
  }

  const resp = await fetch(API_URL + "/auth/validate", {
    method: "GET",
    headers: {
      access_token: accessToken
    }
  });

  return resp.ok;
}
