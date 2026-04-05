<script lang="ts">
  import type { LogColumn } from "$lib/column";
  import type { EntityState } from "$lib/entity.svelte.js";
  import type { SkillState } from "$lib/skill.svelte.js";
  import Back from "./Back.svelte";
  import { getSortedBreakdownColumns } from "./PlayerBreakdownColumns.svelte";
  import QuickTooltip from "./QuickTooltip.svelte";

  let { entityState, handleRightClick }: { entityState: EntityState; handleRightClick: () => void } = $props();
  let columns = $derived(getSortedBreakdownColumns(entityState.isSupport));

  function isActiveSort(columnDef: LogColumn<EntityState, SkillState>): boolean {
    if (entityState.skillSort === "stagger") return columnDef.headerText === "STAG";
    if (entityState.skillSort === "buffed" && entityState.isSupport) return columnDef.headerText === "bDMG";
    return columnDef.headerText === "DMG";
  }

  function handleClick(columnDef: LogColumn<EntityState, SkillState>): void {
    if (isActiveSort(columnDef)) return;
    const isStaggerCol = columnDef.headerText === "STAG" && entityState.anyStagger;
    const isDmgCol = columnDef.headerText === "DMG" || columnDef.headerText === "bDMG";
    if (isStaggerCol) {
      entityState.skillSort = "stagger";
    } else if (isDmgCol && entityState.skillSort === "stagger") {
      entityState.skillSort = "damage";
    } else if (isDmgCol && entityState.isSupport) {
      entityState.skillSort = entityState.skillSort === "buffed" ? "damage" : "buffed";
    }
  }

  function isToggleable(columnDef: LogColumn<EntityState, SkillState>): boolean {
    return (
      (columnDef.headerText === "STAG" && entityState.anyStagger) ||
      columnDef.headerText === "DMG" ||
      columnDef.headerText === "bDMG"
    );
  }
</script>

<th class="w-7 px-2 font-normal">
  <Back {handleRightClick} />
</th>
<th class="w-14 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#each columns as columnDef (columnDef.headerText)}
  {#if columnDef.show(entityState)}
    <th
      class="font-normal {columnDef.width ?? 'w-12'} {isToggleable(columnDef)
        ? 'cursor-pointer underline underline-offset-2'
        : ''}"
      style={isActiveSort(columnDef) ? `background-color: rgb(from ${entityState.color} r g b / 0.10)` : ""}
      onclick={() => handleClick(columnDef)}
    >
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
