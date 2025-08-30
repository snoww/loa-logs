import { addToast } from "$lib/components/Toaster.svelte";
import { raidGates } from "$lib/constants/encounters";
import { settings } from "$lib/stores.svelte";
import type { Encounter } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import pako from "pako";
import { uploadError, uploadTokenError } from "./toasts";

export const API_URL = "https://api.snow.xyz";
// export const API_URL = "http://localhost:5180";

export async function uploadLog(id: number | string, encounter: Encounter, showToast = true, bulk = false) {
  if (
    !encounter.cleared ||
    !encounter.bossOnlyDamage ||
    !encounter.difficulty ||
    !Object.hasOwn(raidGates, encounter.currentBossName)
  ) {
    if (showToast && !bulk) {
      addToast(uploadError("Log not supported for upload", id));
    }

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

    await invoke("write_log", {
      message: `couldn't upload encounter ${id} (${encounter.currentBossName}) - error: ${error}`
    });
    return;
  }

  // handle response
  const body = await resp.json();
  // failed server side encounter validation
  if (body.error) {
    if (body.duplicate) {
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
    if (showToast && !body.error.includes("Boss not supported")) addToast(uploadError(body.error, id));
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

  const resp = await fetch(API_URL + "/auth/validate", {
    method: "GET",
    headers: {
      access_token: accessToken
    }
  });

  return resp.ok;
}
