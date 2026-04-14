<script lang="ts">
  import LiveDamageMeter from "./LiveDamageMeter.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import type { Encounter, EncounterEvent, PartyEvent } from "$lib/types";
  import { uploadLog } from "$lib/utils/sync";
  import {
    adminAlert,
    bannedEvent,
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
    onBannedEvent,
    onClearEncounter
  } from "$lib/api";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  let enc = new EncounterState(undefined, true);
  let time = $state(+Date.now());
  let unsubscribe: (() => void) | null = null;
  let pendingTimeouts: ReturnType<typeof setTimeout>[] = [];

  function trackTimeout(fn: () => void, ms: number) {
    const id = setTimeout(() => {
      pendingTimeouts = pendingTimeouts.filter((t) => t !== id);
      fn();
    }, ms);
    pendingTimeouts.push(id);
    return id;
  }

  onMount(() => {
    const interval = setInterval(() => {
      if (misc.raidInProgress && !misc.paused) {
        time = +Date.now();
      }
    }, 1000);

    listenEvents().then((unsub) => {
      unsubscribe = unsub;
    });

    return () => {
      unsubscribe?.();
      clearInterval(interval);
      for (const id of pendingTimeouts) clearTimeout(id);
      pendingTimeouts = [];
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

    handle = await onInvalidDamage(() => {
      misc.missingInfo = true;
    });
    handles.push(handle);

    handle = await onZoneChange((event) => {
      misc.raidInProgress = false;
      misc.missingInfo = false;
      if (!event.payload) {
        addToast(zoneChange);
      }
      trackTimeout(() => {
        misc.raidInProgress = true;
      }, 6000);
    });
    handles.push(handle);

    handle = await onRaidStart(() => {
      misc.raidInProgress = true;
    });
    handles.push(handle);

    handle = await onResetEncounter(() => {
      // just need to trigger an update
      misc.reset = !misc.reset;
      addToast(resetting);
    });
    handles.push(handle);

    handle = await onPauseEncounter(() => {
      if (misc.paused) {
        addToast(pausing);
      } else {
        addToast(resuming);
      }
    });
    handles.push(handle);

    handle = await onSaveEncounter(() => {
      addToast(manualSave);
      trackTimeout(() => {
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

    handle = await onAdmin(() => {
      addToast(adminAlert);
    });
    handles.push(handle);

    handle = await onBannedEvent(() => {
      addToast(bannedEvent);
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

  $effect(() => {
    if (enc.encounter && enc.encounter.fightStart) {
      enc.duration = time - enc.encounter.fightStart;
    } else {
      enc.duration = 0;
    }
  });

  $effect(() => {
    let hideTimeout: ReturnType<typeof setTimeout> | undefined;
    if (settings.app.general.autoShow && !settings.app.general.mini) {
      const appWindow = getCurrentWebviewWindow();
      if (misc.raidInProgress && enc.encounter?.currentBossName) {
        appWindow.show();
      } else {
        // hide with delay
        hideTimeout = setTimeout(() => {
          if (!enc.encounter) {
            appWindow.hide();
          }
        }, settings.app.general.autoHideDelay);
      }
    }
    return () => {
      if (hideTimeout) clearTimeout(hideTimeout);
    };
  });
</script>

<LiveDamageMeter {enc} />
