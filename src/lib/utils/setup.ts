import { setBlur } from "$lib/api";
import { settings } from "$lib/stores.svelte";
import { invoke } from "@tauri-apps/api/core";

export async function setup() {
  // disable blur on windows 11

  const userAgentData = (navigator as any).userAgentData;

  if (userAgentData && userAgentData.platform === "Windows") {
    let ua = await userAgentData.getHighEntropyValues(["platformVersion"]);
    const majorPlatformVersion = Number(ua.platformVersion.split(".")[0]);
    if (majorPlatformVersion >= 13) {
      settings.app.general.isWin11 = true;
      setBlur(settings.app.general.blurWin11);
    } else if (settings.app.general.blur) {
      setBlur(settings.app.general.blur);
    }
  }
}
