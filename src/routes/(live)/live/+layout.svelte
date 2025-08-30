<script lang="ts">
  import Toaster from "$lib/components/Toaster.svelte";
  import { settings, type LogSettings } from "$lib/stores.svelte";
  import { setup } from "$lib/utils/setup";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import { emit } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { info } from '@tauri-apps/plugin-log';
  import { getSettings } from "$lib/api";
  import { unregisterAll as unregisterAllShortcuts } from "@tauri-apps/plugin-global-shortcut";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    setup();
    (async () => {
      await info("setting up live meter");

      let data = await getSettings();

      if (data) {
        settings.app = data;
      }

      if (settings.app.general.bossOnlyDamageDefaultOn && !settings.app.general.bossOnlyDamage) {
        settings.app.general.bossOnlyDamage = true;
        await emit("boss-only-damage-request", true);
      }

      await info("finished meter setup");
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
      await unregisterAllShortcuts();
      await registerShortcuts();
    })();
  });

  $effect.pre(() => {
    if (settings.app.general.mini) {
      (async () => {
        const appWindow = getCurrentWindow();
        appWindow.hide();
      })()
    }
  });
</script>

<Toaster />
<div
  class="live-meter h-screen min-h-screen overflow-hidden {settings.app.general.transparent
    ? 'bg-neutral-900/25'
    : 'bg-neutral-900/95'}"
  class:hidden={settings.app.general.mini}
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
