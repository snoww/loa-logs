<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EntityState } from "$lib/entity.svelte.js";
  import Back from "./Back.svelte";
  import { playerBreakdownColumns } from "./PlayerBreakdownColumns.svelte";

  let { entityState, handleRightClick }: { entityState: EntityState; handleRightClick: () => void } = $props();
</script>

<th class="w-7 px-2 font-normal">
  <Back {handleRightClick} />
</th>
<th class="w-14 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#each playerBreakdownColumns as columnDef}
  {#if columnDef.show(entityState)}
    <th class="font-normal {columnDef.width ? columnDef.width : 'w-12'}">
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
