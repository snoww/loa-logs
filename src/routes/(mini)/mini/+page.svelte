<script lang="ts">
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc } from "$lib/stores.svelte";
  import type { EncounterEvent } from "$lib/types";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import MiniEncounterInfo from "./MiniEncounterInfo.svelte";
  import MiniPlayers from "./MiniPlayers.svelte";

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
        enc.encounter = event.payload;
      });

      events.push(encounterUpdateEvent);
    })();

    return () => {
      clearInterval(interval);
      events.forEach((unlisten) => unlisten());
    };
  });
</script>

<div class="h-full w-full border border-red-500 overflow-hidden select-none flex flex-col gap-2">
  <MiniPlayers {enc} />
  <MiniEncounterInfo {enc} />
</div>
