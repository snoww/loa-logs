<script lang="ts">
  import { customRound } from "$lib/utils";
  import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  interface IdentityEvent {
    timestamp: number;
    gauge1: number;
    gauge2: number;
    gauge3: number;
  }

  let identityEvents = $state<IdentityEvent[]>([]);
  let startTime: number | undefined = $state(undefined);

  onMount(() => {
    let events: Array<UnlistenFn> = [];
    (async () => {
      await emit("emit-details-request");
      let identityEvent = await listen("identity-update", (event: any) => {
        const identityEvent = event.payload as IdentityEvent;
        identityEvent.timestamp = +Date.now();
        if (!startTime) {
          startTime = identityEvent.timestamp;
        }
        identityEvents.push(identityEvent);
      });
      events.push(identityEvent);
    })();

    return () => {
      events.forEach((unlisten) => unlisten());
    };
  });

  onDestroy(() => {
    emit("emit-details-request");
  });
</script>

<div class="h-full overflow-hidden bg-neutral-800/80 px-3 py-1">
  <div class="flex items-center gap-2">
    <div class="text-medium font-bold">Identity Details</div>
    <button
      class="text-neutral-300 hover:text-neutral-200"
      onclick={() => {
        identityEvents = [];
        startTime = undefined;
      }}>reset</button
    >
  </div>
  <div>
    <div class="relative grid select-text grid-cols-[6rem_6rem_6rem_6rem] gap-x-2 py-1">
      <b>Timestamp</b>
      <b class="text-right">Gauge 1</b>
      <b class="text-right">Gauge 2</b>
      <b class="text-right">Gauge 3</b>
    </div>
  </div>

  {#if startTime}
    <div class="overflow-y-auto" style="height: calc(100% - 1.5rem - 1.5rem );">
      <div class="grid select-text grid-cols-[6rem_6rem_6rem_6rem] gap-x-2 gap-y-1">
        {#each { length: identityEvents.length } as _, i}
          {@const event = identityEvents[identityEvents.length - 1 - i]}
          <p class="font-mono">+{customRound((event.timestamp - startTime) / 1000, 2)}s</p>
          <p class="text-right font-mono">{event.gauge1.toLocaleString()}</p>
          <p class="text-right font-mono">{event.gauge2.toLocaleString()}</p>
          <p class="text-right font-mono">{event.gauge3.toLocaleString()}</p>
        {/each}
      </div>
    </div>
  {/if}
</div>
