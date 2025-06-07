<script lang="ts">
  import { IconScreenshare, IconScreenshareOff } from "$lib/icons";
  import { misc } from "$lib/stores.svelte.js";
  import { startHosting, stopHosting } from "$lib/utils/live.svelte.js";
  import QuickTooltip from "./QuickTooltip.svelte";

  let working = $state(false);

  async function beginLiveSharing() {
    if (working) return;

    working = true;
    const id = await startHosting();
    navigator.clipboard.writeText(`https://live.lostark.bible/${id}`);
    working = false;
  }

  async function stopLiveSharing() {
    if (working) return;

    working = true;
    await stopHosting();
    working = false;
  }
</script>

{#if misc.liveConnectionListening}
  <button onclick={stopLiveSharing} aria-label="Stop Live Sharing" class="group">
    <QuickTooltip tooltip="Stop Live Sharing">
      <IconScreenshareOff class="group-hover:text-accent-500/80 size-4.5 {working ? 'opacity-40' : 'animate-pulse'}" />
    </QuickTooltip>
  </button>
{:else}
  <button onclick={beginLiveSharing} aria-label="Enable Live Sharing" class="group">
    <QuickTooltip tooltip="Start Live Sharing">
      <IconScreenshare class="group-hover:text-accent-500/80 size-4.5 {working ? 'opacity-40' : ''}" />
    </QuickTooltip>
  </button>
{/if}
