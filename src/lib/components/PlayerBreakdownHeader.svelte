<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EntityState } from "$lib/entity.svelte.js";
  import Back from "./Back.svelte";
  import { getSortedBreakdownColumns } from "./PlayerBreakdownColumns.svelte";

  let { entityState, handleRightClick }: { entityState: EntityState; handleRightClick: () => void } = $props();
  let columns = $derived(getSortedBreakdownColumns(entityState.isSupport));
</script>

<th class="w-7 px-2 font-normal">
  <Back {handleRightClick} />
</th>
<th class="w-14 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#each columns as columnDef (columnDef.headerText)}
  {#if columnDef.show(entityState)}
    {@const isActiveSort =
      entityState.isSupport &&
      ((entityState.sortByBuffed && columnDef.isSort) ||
        (!entityState.sortByBuffed && columnDef.headerText === "DMG"))}
    {@const isToggleable =
      entityState.isSupport && (columnDef.headerText === "bDMG" || columnDef.headerText === "DMG")}
    <th
      class="font-normal {columnDef.width ? columnDef.width : 'w-12'} {isActiveSort
        ? 'text-accent-400 underline underline-offset-2'
        : ''} {isToggleable ? 'cursor-pointer' : ''}"
      onclick={isToggleable ? () => (entityState.sortByBuffed = !entityState.sortByBuffed) : undefined}
    >
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
