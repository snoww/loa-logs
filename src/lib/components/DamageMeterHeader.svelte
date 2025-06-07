<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { logColumns } from "./DamageMeterColumns.svelte";

  let { enc }: { enc: EncounterState } = $props();
</script>

<th class="w-20 px-2 text-left font-normal"></th>
<th class="w-full"></th>

{#each logColumns as columnDef}
  {#if columnDef.show(enc)}
    <th class="font-normal {columnDef.width ? columnDef.width : 'w-12'}">
      <QuickTooltip tooltip={columnDef.headerTooltip}>{columnDef.headerText}</QuickTooltip>
    </th>
  {/if}
{/each}
