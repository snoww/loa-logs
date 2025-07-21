<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import { MeterTab } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";

  let { tab = $bindable() }: { tab: MeterTab } = $props();
</script>

{#snippet meterTab(name: string, t: MeterTab)}
  <button class="rounded-xs shrink-0 px-1.5 transition {tab === t ? 'bg-accent-500/40' : ''}" onclick={() => (tab = t)}>
    {name}
  </button>
{/snippet}
<div class="flex h-6 select-none items-center justify-between bg-neutral-800/70 px-1 text-neutral-300">
  <div class="flex h-full items-center overflow-x-scroll text-xs">
    {@render meterTab("DPS", MeterTab.DAMAGE)}
    {@render meterTab("PARTY", MeterTab.PARTY_BUFFS)}
    {@render meterTab("SELF", MeterTab.SELF_BUFFS)}
    {@render meterTab("TANK", MeterTab.TANK)}
    {@render meterTab("BOSS", MeterTab.BOSS)}
    {#if settings.app.general.showDetails}
      {@render meterTab("DETAILS", MeterTab.DETAILS)}
    {/if}
  </div>
  <div class="flex items-center gap-1 px-1 tracking-tighter">
    <div class="text-xs">LOA Logs</div>
    <div class="text-xs text-neutral-500">
      {#await getVersion()}
        v
      {:then version}
        v{version}
      {/await}
    </div>
  </div>
</div>
