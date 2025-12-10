<script lang="ts">
  import { getSettings, setBossOnlyDamage, writeLog } from "$lib/api";
  import Toaster from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { setup } from "$lib/utils/setup";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    setup();
    (async () => {
      await writeLog("setting up live meter");
      let data = await getSettings();
      if (data) {
        settings.app = data;
      }

      await writeLog("finished meter setup");
    })();
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
      await registerShortcuts();
    })();
  });

  $effect.pre(() => {
    if (settings.app.general.mini) {
      getCurrentWebviewWindow().hide();
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
