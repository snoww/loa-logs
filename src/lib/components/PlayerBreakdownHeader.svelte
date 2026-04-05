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
    {@const isStaggerSortable = columnDef.headerText === "STAG" && entityState.anyStagger}
    {@const isDmgCol = columnDef.headerText === "DMG" || (entityState.isSupport && columnDef.headerText === "bDMG")}
    {@const isToggleable = isStaggerSortable || isDmgCol}
    {@const isActiveSort = entityState.sortByStagger
      ? columnDef.headerText === "STAG"
      : entityState.isSupport
        ? ((entityState.sortByBuffed && columnDef.isSort) || (!entityState.sortByBuffed && columnDef.headerText === "DMG"))
        : columnDef.headerText === "DMG"}
    {@const handleClick = isActiveSort
      ? undefined
      : isStaggerSortable
        ? () => { entityState.sortByStagger = true; }
        : isDmgCol && entityState.sortByStagger
          ? () => { entityState.sortByStagger = false; }
          : isDmgCol && entityState.isSupport
            ? () => { entityState.sortByBuffed = !entityState.sortByBuffed; }
            : undefined}
    <th
      class="font-normal {columnDef.width ? columnDef.width : 'w-12'} {isActiveSort
        ? 'underline underline-offset-2'
        : isToggleable
          ? 'underline underline-offset-2'
          : ''} {isToggleable ? 'cursor-pointer' : ''}"
      style={isActiveSort ? `background-color: rgb(from ${entityState.color} r g b / 0.15)` : ''}
      onclick={handleClick}
    >
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
