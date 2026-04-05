<script lang="ts">
  import type { LogColumn } from "$lib/column";
  import { type EntityState, SkillSort } from "$lib/entity.svelte.js";
  import type { SkillState } from "$lib/skill.svelte.js";
  import Back from "./Back.svelte";
  import { getSortedBreakdownColumns } from "./PlayerBreakdownColumns.svelte";
  import QuickTooltip from "./QuickTooltip.svelte";

  let { entityState, handleRightClick }: { entityState: EntityState; handleRightClick: () => void } = $props();
  let columns = $derived(getSortedBreakdownColumns(entityState.isSupport));

  function isActiveSort(columnDef: LogColumn<EntityState, SkillState>): boolean {
    if (entityState.skillSort === SkillSort.Stagger) {
      return columnDef.headerText === "STAG";
    }
    if (entityState.isSupport) {
      const sortingByBuffed = entityState.skillSort === SkillSort.Buffed && !!columnDef.isSort;
      const sortingByDamage = entityState.skillSort === SkillSort.Damage && columnDef.headerText === "DMG";
      return sortingByBuffed || sortingByDamage;
    }
    return columnDef.headerText === "DMG";
  }

  function handleClick(columnDef: LogColumn<EntityState, SkillState>): (() => void) | undefined {
    if (isActiveSort(columnDef)) return undefined;
    const isStaggerCol = columnDef.headerText === "STAG" && entityState.anyStagger;
    const isDmgCol = columnDef.headerText === "DMG" || (entityState.isSupport && columnDef.headerText === "bDMG");
    if (isStaggerCol) return () => { entityState.skillSort = SkillSort.Stagger; };
    if (isDmgCol && entityState.skillSort === SkillSort.Stagger) return () => { entityState.skillSort = SkillSort.Damage; };
    if (isDmgCol && entityState.isSupport) return () => {
      if (entityState.skillSort === SkillSort.Buffed) {
        entityState.skillSort = SkillSort.Damage;
      } else {
        entityState.skillSort = SkillSort.Buffed;
      }
    };
    return undefined;
  }

  function isToggleable(columnDef: LogColumn<EntityState, SkillState>): boolean {
    return (columnDef.headerText === "STAG" && entityState.anyStagger) ||
           columnDef.headerText === "DMG" ||
           (entityState.isSupport && columnDef.headerText === "bDMG");
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
      class="font-normal {columnDef.width ?? 'w-12'} {isToggleable(columnDef) ? 'cursor-pointer underline underline-offset-2' : ''}"
      style={isActiveSort(columnDef) ? `background-color: rgb(from ${entityState.color} r g b / 0.15)` : ''}
      onclick={handleClick(columnDef)}
    >
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
