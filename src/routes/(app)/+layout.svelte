<script lang="ts">
  import { goto } from "$app/navigation";
  import { getSettings, loadEncounter, onClearEncounter, onLatestEncounter, onRedirectUrl, writeLog } from "$lib/api";
  // import BetaWelcome from "$lib/components/BetaWelcome.svelte";
  import NinevehWelcome from "$lib/components/NinevehWelcome.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
  import UpdateAvailable from "$lib/components/UpdateAvailable.svelte";
  import { settings } from "$lib/stores.svelte";
  import { checkForUpdate } from "$lib/utils";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import { uploadLog } from "$lib/utils/sync";
  import { getVersion } from "@tauri-apps/api/app";
  import { type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onDestroy, onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  let events: Set<UnlistenFn> = new Set();

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

  onMount(() => {
    (async () => {
      await writeLog("loading app settings");
      let data = await getSettings();
      if (data) {
        settings.app = data;
      }
      await writeLog("finished app settings setup");

      let clearEncounterEvent = await onClearEncounter(async (event) => {
        if (!settings.sync.auto) {
          return;
        }

        let id = event.payload.toString();
        const encounter = await loadEncounter(id);
        await uploadLog(id, encounter, false);
      });
      events.add(clearEncounterEvent);

      let encounterUpdateEvent = await onLatestEncounter(async (event) => {
        await goto("/logs/" + event.payload);
        await showWindow();
      });
      let openUrlEvent = await onRedirectUrl(async (event) => {
        await goto("/" + event.payload);
        await showWindow();
      });

      events.add(encounterUpdateEvent);
      events.add(openUrlEvent);

      let version = await getVersion();
      settings.app.general.betaChannel = version.toLowerCase().includes("beta");
      if (settings.version !== version) {
        settings.version = version;
        goto("/changelog");
        await showWindow();
      }
    })();

    // check for app updates
    const interval = setInterval(() => checkForUpdate(settings.app.general.betaChannel), 60 * 15 * 1000);
    return () => {
      clearInterval(interval);
    };
  });
  onDestroy(() => {
    events.forEach((unlisten) => unlisten());
  });

  async function showWindow() {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.show();
    await appWindow.unminimize();
    await appWindow.setFocus();
  }
</script>

<UpdateAvailable />
<!--{#if settings.app.general.betaChannel}-->
<!--  <BetaWelcome />-->
<!--{/if}-->
<NinevehWelcome />
<Toaster />
<div class="min-h-screen bg-neutral-900 select-none">
  {@render children?.()}
</div>
