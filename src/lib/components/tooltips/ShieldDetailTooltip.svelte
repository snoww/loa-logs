<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { ShieldDetails } from "$lib/types";
  import { abbreviateNumberSplit } from "$lib/utils/numbers";
  import { getSkillIcon } from "$lib/utils/strings";

  interface Props {
    shieldDetails: ShieldDetails;
  }

  let { shieldDetails }: Props = $props();
  let shield = $derived(abbreviateNumberSplit(shieldDetails.total));
</script>

{#snippet tooltip()}
  <div class="-mx-px flex flex-col space-y-1 py-px text-xs font-normal">
    {#each shieldDetails.buffs as buff}
      {@const shield = abbreviateNumberSplit(buff.value)}
      <div class="flex items-center">
        <img src={getSkillIcon(buff.icon)} alt={buff.icon} class="mr-1 size-5 rounded-sm" />
        {shield[0]}<span class="text-3xs text-neutral-300">{shield[1]}</span>
      </div>
    {/each}
  </div>
{/snippet}

<QuickTooltip {tooltip}>
  <div class="relative z-20">
    {shield[0]}<span class="text-3xs text-neutral-300">{shield[1]}</span>
  </div>
</QuickTooltip>
