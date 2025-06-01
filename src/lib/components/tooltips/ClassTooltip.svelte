<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { Entity } from "$lib/types";

  import { getClassIcon } from "$lib/utils";

  const { entity: entity }: { entity: Entity } = $props();
</script>

{#snippet tooltip()}
  <div class="flex">
    {#if entity.arkPassiveActive}
      <div class="mr-1"><span class="text-purple-400">[Ark Passive]</span></div>
    {/if}
    {#if entity.spec}
      <span>{entity.spec} {entity.class}</span>
    {:else}
      <span>{entity.class}</span>
    {/if}
  </div>
{/snippet}

<QuickTooltip {tooltip}>
  <img class="table-cell size-5" src={getClassIcon(entity.classId)} alt={entity.class} />
</QuickTooltip>
