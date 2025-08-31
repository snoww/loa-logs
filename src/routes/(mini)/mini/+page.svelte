<script lang="ts">
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import type { EncounterEvent } from "$lib/types";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import MiniEncounterInfo from "./MiniEncounterInfo.svelte";
  import MiniPlayers from "./MiniPlayers.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  let enc = $derived(new EncounterState(undefined, true));
  let time = $state(+Date.now());
  onMount(() => {
    const interval = setInterval(() => {
      if (misc.raidInProgress) {
        time = +Date.now();
      }
    }, 1000);

    let events: Array<UnlistenFn> = [];
    (async () => {
      let encounterUpdateEvent = await listen("encounter-update", (event: EncounterEvent) => {
        if (settings.app.general.mini) {
          enc.encounter = event.payload;
        }
      });
      let raidStartEvent = await listen("raid-start", () => {
        misc.raidInProgress = true;
      });
      let zoneChangeEvent = await listen("zone-change", () => {
        misc.raidInProgress = false;
        setTimeout(() => {
          misc.raidInProgress = true;
        }, 8000);
      });
      let phaseTransitionEvent = await listen("phase-transition", (event: any) => {
        misc.raidInProgress = false;
      });

      events.push(encounterUpdateEvent, zoneChangeEvent, phaseTransitionEvent, raidStartEvent);
    })();

    return () => {
      clearInterval(interval);
      events.forEach((unlisten) => unlisten());
    };
  });

  $effect(() => {
    if (settings.app.general.autoShow && settings.app.general.mini) {
      const appWindow = getCurrentWebviewWindow();
      if (misc.raidInProgress && enc.encounter?.currentBossName) {
        appWindow.show();
      } else {
        // hide with delay
        setTimeout(() => {
          if (!enc.encounter) {
            appWindow.hide();
          }
        }, settings.app.general.autoHideDelay);
      }
    }
  });

  $effect(() => {
    if (misc.raidInProgress) {
      enc.reset();
    }
  });

  $effect(() => {
    if (enc.encounter && enc.encounter.fightStart) {
      enc.duration = time - enc.encounter.fightStart;
    } else {
      enc.duration = 0;
    }
  });
</script>

<MiniPlayers {enc} />
<MiniEncounterInfo {enc} />
