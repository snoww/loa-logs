<script lang="ts">
  import DamageMeter from "$lib/components/DamageMeter.svelte";
  import { settings, type LogSettings } from "$lib/stores.svelte";
  import { registerShortcuts } from "$lib/utils/settings";
  import { invoke } from "@tauri-apps/api";
  import { emit } from "@tauri-apps/api/event";
  import { unregisterAll } from "@tauri-apps/api/globalShortcut";
  import { appWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  onMount(() => {
    (async () => {
      await invoke("write_log", { message: "setting up live meter" });
      let data = (await invoke("get_settings")) as LogSettings;
      if (data) {
        settings.app = data;
      }

      // updateSettings.set(update);
      if (settings.app.general.alwaysOnTop) {
        await appWindow.setAlwaysOnTop(true);
      } else {
        await appWindow.setAlwaysOnTop(false);
      }

      if (settings.app.general.bossOnlyDamageDefaultOn && !settings.app.general.bossOnlyDamage) {
        settings.app.general.bossOnlyDamage = true;
        await emit("boss-only-damage-request", true);
      }

      // try {
      //   const { shouldUpdate, manifest } = await checkUpdate();
      //   if (shouldUpdate) {
      //     $updateSettings.available = true;
      //     const oldManifest = $updateSettings.manifest;
      //     $updateSettings.manifest = manifest;
      //     if (oldManifest?.version !== $updateSettings.manifest?.version) {
      //       $updateSettings.dismissed = false;
      //     }
      //     $updateSettings.isNotice = manifest?.version.includes("2024");
      //   }
      // } catch (e) {
      //   await invoke("write_log", { message: String(e) });
      // }

      // await registerShortcuts(settings.appSettings.shortcuts);

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

<div
  class="live-meter h-screen overflow-hidden {settings.app.general.transparent
    ? 'bg-zinc-800/[.2]'
    : 'bg-zinc-800 opacity-95'}"
>
  <DamageMeter />
</div>

<style lang="postcss">
  :global(.live-meter *) {
    -ms-overflow-style: none;
    scrollbar-width: auto;
  }
  :global(.live-meter *::-webkit-scrollbar) {
    display: none;
  }
</style>
