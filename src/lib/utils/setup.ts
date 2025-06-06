import { settings } from "$lib/stores.svelte";
import { invoke } from "@tauri-apps/api";

export async function setup() {
  // disable blur on windows 11
  let ua = await navigator.userAgentData.getHighEntropyValues(["platformVersion"]);
  if (navigator.userAgentData.platform === "Windows") {
    const majorPlatformVersion = Number(ua.platformVersion.split(".")[0]);
    if (majorPlatformVersion >= 13) {
      settings.app.general.isWin11 = true;
      if (settings.app.general.blurWin11) {
        await invoke("enable_blur");
      } else {
        await invoke("disable_blur");
      }
    } else if (settings.app.general.blur) {
      await invoke("enable_blur");
    } else {
      await invoke("disable_blur");
    }
  }
}
