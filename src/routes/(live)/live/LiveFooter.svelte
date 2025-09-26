<script lang="ts">
  import { appContext } from "$lib/context.svelte";
  import { settings } from "$lib/stores.svelte";
  import { MeterTab } from "$lib/types";
  import type { MouseEventHandler } from "svelte/elements";

  interface Props {
    tab: MeterTab
  }

  let { tab = $bindable() }: Props = $props();

  const allTabs: { name: string; value: MeterTab }[] = [
    { name: "DPS",     value: MeterTab.DAMAGE },
    { name: "PARTY",   value: MeterTab.PARTY_BUFFS },
    { name: "SELF",    value: MeterTab.SELF_BUFFS },
    { name: "TANK",    value: MeterTab.TANK },
    { name: "BOSS",    value: MeterTab.BOSS },
    { name: "DETAILS", value: MeterTab.DETAILS },
  ];

  const tabs = $derived.by(() => settings.app.general.showDetails
      ? allTabs
      : allTabs.filter(t => t.value !== MeterTab.DETAILS)
  )

  const onTabSelect: MouseEventHandler<HTMLButtonElement> = (event) => {
    const selectedTab = Number(event.currentTarget.dataset.tab);
    tab = selectedTab
  }
</script>

<div class="flex h-6 select-none items-center justify-between bg-neutral-800/70 px-1 text-neutral-300">
  <div class="flex h-full items-center overflow-x-scroll text-xs">
      {#each tabs as { name, value }}
        <button
          type="button"
          data-tab={value}
          data-selected={value == tab || undefined}
          class="rounded-xs shrink-0 px-1.5 transition duration-200 data-[selected]:bg-accent-500/40"
          onclick={onTabSelect}
        >
          {name}
        </button>
    {/each}
  </div>
  <div class="flex items-center gap-1 px-1 tracking-tighter">
    <div class="text-xs">{$appContext.productName}</div>
    <div class="text-xs text-neutral-500">
      {$appContext.version}
    </div>
  </div>
</div>
