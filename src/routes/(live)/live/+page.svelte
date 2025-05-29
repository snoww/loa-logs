<script lang="ts">
  import DamageMeter from "$lib/components/DamageMeter.svelte";
  import { settings } from "$lib/stores.svelte";
  import { registerShortcuts, shortcuts, type LogSettings } from "$lib/utils/settings";
  import { invoke } from "@tauri-apps/api";
  import { emit } from "@tauri-apps/api/event";
  import { register, unregister, unregisterAll } from "@tauri-apps/api/globalShortcut";
  import { appWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  onMount(() => {
    (async () => {
      await invoke("write_log", { message: "setting up live meter" });
      let data = (await invoke("get_settings")) as LogSettings;
      if (data) {
        settings.appSettings = data;
      }

      // updateSettings.set(update);
      if (settings.appSettings.general.alwaysOnTop) {
        await appWindow.setAlwaysOnTop(true);
      } else {
        await appWindow.setAlwaysOnTop(false);
      }

      if (settings.appSettings.general.bossOnlyDamageDefaultOn && !settings.appSettings.general.bossOnlyDamage) {
        settings.appSettings.general.bossOnlyDamage = true;
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
          settings.appSettings.general.isWin11 = true;
          if (settings.appSettings.general.blurWin11) {
            await invoke("enable_blur");
          } else {
            await invoke("disable_blur");
          }
        } else if (settings.appSettings.general.blur) {
          await invoke("enable_blur");
        } else {
          await invoke("disable_blur");
        }
      }

      await invoke("write_log", { message: "finished meter setup" });
    })();
  });

  $effect.pre(() => {
    if (settings.appSettings.general.scale === "1") {
      document.documentElement.style.setProperty("font-size", "medium");
    } else if (settings.appSettings.general.scale === "2") {
      document.documentElement.style.setProperty("font-size", "large");
    } else if (settings.appSettings.general.scale === "3") {
      document.documentElement.style.setProperty("font-size", "x-large");
    } else if (settings.appSettings.general.scale === "0") {
      document.documentElement.style.setProperty("font-size", "small");
    }
  });

  $effect.pre(() => {
    settings.appSettings.shortcuts.hideMeter;
    settings.appSettings.shortcuts.showLogs;
    settings.appSettings.shortcuts.showLatestEncounter;
    settings.appSettings.shortcuts.resetSession;
    settings.appSettings.shortcuts.pauseSession;
    settings.appSettings.shortcuts.manualSave;
    settings.appSettings.shortcuts.disableClickthrough;

    (async () => {
      await unregisterAll();
      await registerShortcuts();
    })();
  });
</script>

<div
  class="live-meter h-screen overflow-hidden {settings.appSettings.general.transparent
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
