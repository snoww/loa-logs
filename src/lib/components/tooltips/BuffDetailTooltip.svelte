<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { BuffDetails } from "$lib/types";

  import { getSkillIcon } from "$lib/utils";

  interface Props {
    buffDetails: BuffDetails;
  }

  let { buffDetails: buffDetails }: Props = $props();
</script>

{#snippet tooltip()}
  <div class="-mx-px flex flex-col space-y-1 py-px text-xs font-normal">
    {#each buffDetails.buffs as buff}
      {#if buff.sourceIcon}
        <div class="flex items-center">
          <img src={getSkillIcon(buff.sourceIcon)} alt={buff.sourceIcon} class="mr-1 size-5 rounded-sm" />
          {#if buff.bonus}
            [{buff.bonus}<span class="text-xxs text-neutral-300">%</span>]
          {/if}
          {buff.percentage}<span class="text-xxs text-neutral-300">%</span>
        </div>
      {:else}
        <div class="flex items-center">
          <img src={getSkillIcon(buff.icon)} alt={buff.icon} class="mr-1 size-5 rounded-sm" />
          {buff.percentage}<span class="text-xxs text-neutral-300">%</span>
        </div>
      {/if}
    {/each}
  </div>
{/snippet}

<QuickTooltip {tooltip}>
  <div class="relative z-20">
    {buffDetails.percentage}<span class="text-xxs text-neutral-300">%</span>
  </div>
</QuickTooltip>
