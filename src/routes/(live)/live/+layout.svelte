<script lang="ts">
  import Toaster from "$lib/components/Toaster.svelte";
  import { settings, type LogSettings } from "$lib/stores.svelte";
  import { registerShortcuts } from "$lib/utils/settings";
  import { invoke } from "@tauri-apps/api";
  import { emit } from "@tauri-apps/api/event";
  import { unregisterAll } from "@tauri-apps/api/globalShortcut";
  import { appWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    (async () => {
      await invoke("write_log", { message: "setting up live meter" });
      let data = (await invoke("get_settings")) as LogSettings;
      if (data) {
        settings.app = data;
      }

      if (settings.app.general.alwaysOnTop) {
        await appWindow.setAlwaysOnTop(true);
      } else {
        await appWindow.setAlwaysOnTop(false);
      }

      if (settings.app.general.bossOnlyDamageDefaultOn && !settings.app.general.bossOnlyDamage) {
        settings.app.general.bossOnlyDamage = true;
        await emit("boss-only-damage-request", true);
      }

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

      await invoke("write_log", { message: "finished meter setup" });
    })();
  });

  $effect.pre(() => {
    if (settings.app.general.scale === "1") {
      document.documentElement.style.setProperty("font-size", "medium");
    } else if (settings.app.general.scale === "2") {
      document.documentElement.style.setProperty("font-size", "large");
    } else if (settings.app.general.scale === "3") {
      document.documentElement.style.setProperty("font-size", "x-large");
    } else if (settings.app.general.scale === "0") {
      document.documentElement.style.setProperty("font-size", "small");
    }
  });

  $effect.pre(() => {
    settings.app.shortcuts.hideMeter;
    settings.app.shortcuts.showLogs;
    settings.app.shortcuts.showLatestEncounter;
    settings.app.shortcuts.resetSession;
    settings.app.shortcuts.pauseSession;
    settings.app.shortcuts.manualSave;
    settings.app.shortcuts.disableClickthrough;

    (async () => {
      await unregisterAll();
      await registerShortcuts();
    })();
  });
</script>

<Toaster />
<div
  class="live-meter h-screen min-h-screen overflow-hidden {settings.app.general.transparent
    ? 'bg-neutral-900/25'
    : 'bg-neutral-900/95'}"
>
  {@render children?.()}
</div>

<style lang="postcss">
  :global(.live-meter *, .noscroll) {
    -ms-overflow-style: none;
    scrollbar-width: auto;
  }
  :global(.live-meter *::-webkit-scrollbar, .noscroll::-webkit-scrollbar) {
    display: none;
  }
</style>
