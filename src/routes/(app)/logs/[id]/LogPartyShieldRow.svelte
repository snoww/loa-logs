<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import type { Entity, ShieldDetails } from "$lib/types";
  import ClassTooltip from "$lib/components/tooltips/ClassTooltip.svelte";
  import ShieldDetailTooltip from "$lib/components/tooltips/ShieldDetailTooltip.svelte";
  import { abbreviateNumberSplit } from "$lib/utils";

  interface Props {
    enc: EncounterState;
    player: Entity;
    playerShields: Array<ShieldDetails>;
    percentage: number;
  }

  let { enc, player, playerShields, percentage }: Props = $props();
  let entityState = $derived(new EntityState(player, enc));

  let totalShield = $derived(playerShields.reduce((acc, buff) => acc + buff.total, 0));
  let totalShieldStr = $derived(abbreviateNumberSplit(totalShield));
</script>

<td class="pl-1">
  <ClassTooltip entity={player} />
</td>
<td colspan="2">
  <div class="flex truncate">
    <QuickTooltip tooltip={entityState.name} class="truncate">
      {entityState.name}
    </QuickTooltip>
  </div>
</td>
<td class="text-xxs px-1 text-center">
  {totalShieldStr[0]}<span class="text-xxs text-neutral-300">{totalShieldStr[1]}</span>
</td>
{#if playerShields.length > 0}
  {#each playerShields as shield (shield.id)}
    <td class="text-xxs px-1 text-center">
      {#if shield.total}
        <ShieldDetailTooltip shieldDetails={shield} />
      {/if}
    </td>
  {/each}
{/if}
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / 0.6); width: {percentage}%"
></td>
