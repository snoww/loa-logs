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
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import {
    loadEncounter,
    onEncounterUpdate,
    onPartyUpdate,
    onInvalidDamage,
    onZoneChange,
    onRaidStart,
    onResetEncounter,
    onPauseEncounter,
    onSaveEncounter,
    onPhaseTransition,
    onAdmin,
    onClearEncounter
  } from "$lib/api";
  import type { UnlistenFn } from "@tauri-apps/api/event";

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

  async function listenEvents() {
    let handles: Array<UnlistenFn> = [];

    let handle = await onEncounterUpdate((event) => {
      if (!settings.app.general.mini) {
        enc.encounter = event.payload;
      }
    });
    handles.push(handle);

    handle = await onPartyUpdate((event) => {
      if (event.payload) {
        enc.partyInfo = event.payload;
      }
    });
    handles.push(handle);

    handle = await onInvalidDamage((_) => {
      misc.missingInfo = true;
    });
    handles.push(handle);

    handle = await onZoneChange((_) => {
      misc.raidInProgress = false;
      addToast(zoneChange);
      setTimeout(() => {
        misc.raidInProgress = true;
      }, 6000);
    });
    handles.push(handle);

    handle = await onRaidStart((_) => {
      misc.raidInProgress = true;
    });
    handles.push(handle);

    handle = await onResetEncounter((_) => {
      // just need to trigger an update
      misc.reset = !misc.reset;
      addToast(resetting);
    });
    handles.push(handle);

    handle = await onPauseEncounter((_) => {
      if (misc.paused) {
        addToast(pausing);
      } else {
        addToast(resuming);
      }
    });
    handles.push(handle);

    handle = await onSaveEncounter((_) => {
      addToast(manualSave);
      setTimeout(() => {
        misc.reset = !misc.reset;
      }, 1000);
    });
    handles.push(handle);

    handle = await onPhaseTransition((event) => {
      let phaseCode = event.payload;

      if (phaseCode === 1) {
        addToast(bossDead);
      } else if (phaseCode === 2 && misc.raidInProgress) {
        addToast(raidClear);
      } else if (phaseCode === 4 && misc.raidInProgress) {
        addToast(raidWipe);
      }
      misc.raidInProgress = false;
    });
    handles.push(handle);

    handle = await onAdmin((_) => {
      addToast(adminAlert);
    });
    handles.push(handle);

    handle = await onClearEncounter(async (event) => {
      if (!settings.sync.auto) {
        return;
      }

      let id = event.payload.toString();
      const encounter = await loadEncounter(id);
      await uploadLog(id, encounter, false);
    });
    handles.push(handle);

    return () => {
      for (const unlisten of handles) {
        unlisten();
      }
    };
  }

  async function onLoad() {
    unsubscribe = await listenEvents();
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
