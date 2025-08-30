<script lang="ts">
  import LiveDamageMeter from "./LiveDamageMeter.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import type { Encounter, EncounterEvent, PartyEvent } from "$lib/types";
  import { uploadLog } from "$lib/utils/sync";
  import {
    adminAlert,
    bossDead,
    manualSave,
    pausing,
    raidClear,
    raidWipe,
    resetting,
    resuming,
    zoneChange
  } from "$lib/utils/toasts";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";

  let enc = $derived(new EncounterState(undefined, true));
  let time = $state(+Date.now());

  onMount(() => {
    const interval = setInterval(() => {
      if (misc.raidInProgress && !misc.paused) {
        time = +Date.now();
      }
    }, 1000);

    let events: Array<UnlistenFn> = [];

    (async () => {
      let encounterUpdateEvent = await listen("encounter-update", (event: EncounterEvent) => {
        // console.log(+Date.now(), event.payload);
        if (!settings.app.general.mini) {
          enc.encounter = event.payload;
        }
      });
      let partyUpdateEvent = await listen("party-update", (event: PartyEvent) => {
        if (event.payload) {
          enc.partyInfo = event.payload;
        }
      });
      let invalidDamageEvent = await listen("invalid-damage", () => {
        misc.missingInfo = true;
      });
      let zoneChangeEvent = await listen("zone-change", () => {
        misc.raidInProgress = false;
        addToast(zoneChange);
        setTimeout(() => {
          misc.raidInProgress = true;
        }, 6000);
      });
      let raidStartEvent = await listen("raid-start", () => {
        misc.raidInProgress = true;
      });
      let resetEncounterEvent = await listen("reset-encounter", () => {
        // just need to trigger an update
        misc.reset = !misc.reset;
        addToast(resetting);
      });
      let pauseEncounterEvent = await listen("pause-encounter", () => {
        if (misc.paused) {
          addToast(pausing);
        } else {
          addToast(resuming);
        }
      });
      let saveEncounterEvent = await listen("save-encounter", () => {
        addToast(manualSave);
        setTimeout(() => {
          misc.reset = !misc.reset;
        }, 1000);
      });
      let phaseTransitionEvent = await listen("phase-transition", (event: any) => {
        let phaseCode = event.payload;
        // console.log(Date.now() + ": phase transition event: ", event.payload)
        if (phaseCode === 1) {
          addToast(bossDead);
        } else if (phaseCode === 2 && misc.raidInProgress) {
          addToast(raidClear);
        } else if (phaseCode === 4 && misc.raidInProgress) {
          addToast(raidWipe);
        }
        misc.raidInProgress = false;
      });
      let adminErrorEvent = await listen("admin", () => {
        addToast(adminAlert);
      });
      let clearEncounterEvent = await listen("clear-encounter", async (event: any) => {
        if (!settings.sync.auto) {
          return;
        }

        let id = event.payload.toString();
        const encounter = (await invoke("load_encounter", { id })) as Encounter;
        await uploadLog(id, encounter, false);
      });

      events.push(
        encounterUpdateEvent,
        partyUpdateEvent,
        invalidDamageEvent,
        zoneChangeEvent,
        raidStartEvent,
        resetEncounterEvent,
        pauseEncounterEvent,
        saveEncounterEvent,
        phaseTransitionEvent,
        adminErrorEvent,
        clearEncounterEvent
      );
    })();

    return () => {
      events.forEach((f) => f());
      clearInterval(interval);
    };
  });

  $effect(() => {
    if (enc.encounter && enc.encounter.fightStart) {
      enc.duration = time - enc.encounter.fightStart;
    } else {
      enc.duration = 0;
    }
  });

  $effect(() => {
    if (settings.app.general.autoShow && !settings.app.general.mini) {
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
</script>

<LiveDamageMeter {enc} />
