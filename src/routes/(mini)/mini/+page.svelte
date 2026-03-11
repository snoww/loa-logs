<script lang="ts">
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import type { EncounterEvent } from "$lib/types";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import MiniEncounterInfo from "./MiniEncounterInfo.svelte";
  import MiniPlayers from "./MiniPlayers.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onEncounterUpdate, onPhaseTransition, onRaidStart, onZoneChange } from "$lib/api";

  let enc = $derived(new EncounterState(undefined, true));
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
      if (misc.raidInProgress) {
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
      if (settings.app.general.mini) {
        enc.encounter = event.payload;
      }
    });
    handles.push(handle);

    handle = await onRaidStart(() => {
      misc.raidInProgress = true;
    });
    handles.push(handle);

    handle = await onZoneChange(() => {
      misc.raidInProgress = false;
      misc.missingInfo = false;
      trackTimeout(() => {
        misc.raidInProgress = true;
      }, 8000);
    });
    handles.push(handle);

    handle = await onPhaseTransition((_) => {
      misc.raidInProgress = false;
    });
    handles.push(handle);

    return () => {
      for (const unlisten of handles) {
        unlisten();
      }
    };
  }

  $effect(() => {
    let hideTimeout: ReturnType<typeof setTimeout> | undefined;
    if (settings.app.general.autoShow && settings.app.general.mini) {
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
