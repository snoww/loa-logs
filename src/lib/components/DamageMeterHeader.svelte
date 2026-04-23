<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState, PlayerSort } from "$lib/encounter.svelte.js";
  import { logColumns } from "./DamageMeterColumns.svelte";

  let { enc, sortable = true }: { enc: EncounterState; sortable?: boolean } = $props();

  const columnSortMap: Record<string, PlayerSort> = {
    DMG: "damage",
    rDPS: "rdps",
    STAG: "stagger"
  };

  function getSort(headerText: string): PlayerSort | undefined {
    return columnSortMap[headerText];
  }

  function handleClick(headerText: string): void {
    const sort = getSort(headerText);
    if (sort) enc.playerSort = sort;
  }
</script>

<th class="w-20 px-2 text-left font-normal"></th>
<th class="w-full"></th>

{#each logColumns as columnDef}
  {#if columnDef.show(enc)}
    {@const sort = sortable ? getSort(columnDef.headerText) : undefined}
    <th
      class="font-normal {columnDef.width ?? 'w-12'} {sort
        ? 'cursor-pointer underline underline-offset-2'
        : ''} {sort && enc.playerSort === sort ? 'bg-white/5' : ''}"
      onclick={sort ? () => handleClick(columnDef.headerText) : null}
    >
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
