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
  import { loadEncounter, onLiveEvent, type LiveEvent } from "$lib/api";

  let enc = $derived(new EncounterState(undefined, true));
  let time = $state(+Date.now());
  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    const interval = setInterval(() => {
      if (misc.raidInProgress && !misc.paused) {
        time = +Date.now();
      }
    }, 1000);

    onLoad();

    return () => {
      unsubscribe && unsubscribe();
      clearInterval(interval);
    };
  });

  async function onEvent(event: LiveEvent) {
    switch (event.type) {
      case "encounter-update":
        if (!settings.app.general.mini) {
          enc.encounter = event.payload;
        }
        break;
      case "party-update":
        if (event.payload) {
          enc.partyInfo = event.payload;
        }
        break;
      case "invalid-damage":
        misc.missingInfo = true;
        break;
      case "zone-change":
        misc.raidInProgress = false;
        addToast(zoneChange);
        setTimeout(() => {
          misc.raidInProgress = true;
        }, 6000);
        break;
      case "raid-start":
        misc.raidInProgress = true;
        break;
      case "reset-encounter":
        // just need to trigger an update
        misc.reset = !misc.reset;
        addToast(resetting);
        break;
      case "pause-encounter":
        if (misc.paused) {
          addToast(pausing);
        } else {
          addToast(resuming);
        }
        break;
      case "save-encounter":
        addToast(manualSave);
        setTimeout(() => {
          misc.reset = !misc.reset;
        }, 1000);
        break;
      case "phase-transition":
        let phaseCode = event.payload;

        if (phaseCode === 1) {
          addToast(bossDead);
        } else if (phaseCode === 2 && misc.raidInProgress) {
          addToast(raidClear);
        } else if (phaseCode === 4 && misc.raidInProgress) {
          addToast(raidWipe);
        }
        misc.raidInProgress = false;
        break;
      case "admin":
        addToast(adminAlert);
        break;
      case "clear-encounter":
        if (!settings.sync.auto) {
          return;
        }

        let id = event.payload.toString();
        const encounter = await loadEncounter(id);
        await uploadLog(id, encounter, false);
        break;
    }
  }

  async function onLoad() {
    unsubscribe = await onLiveEvent(onEvent);
  }

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
