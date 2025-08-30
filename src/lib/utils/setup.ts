import { disableBlur, enableBlur } from "$lib/api";
import { settings } from "$lib/stores.svelte";

export async function setup() {
  // disable blur on windows 11
  if ("userAgentData" in navigator
    && navigator.userAgentData
    && navigator.userAgentData.platform === "Windows") {
    let ua = await navigator.userAgentData.getHighEntropyValues(["platformVersion"]);

    if(!ua.platformVersion) {
      return;
    }

    const majorPlatformVersion = Number(ua.platformVersion.split(".")[0]);
    if (majorPlatformVersion >= 13) {
      settings.app.general.isWin11 = true;
      if (settings.app.general.blurWin11) {
        await enableBlur();
      } else {
        await disableBlur()
      }
    } else if (settings.app.general.blur) {
      await enableBlur();
    } else {
      await disableBlur()
    }
  }
}
