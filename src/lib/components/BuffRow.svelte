<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { BuffDetails, type Entity, type StatusEffect } from "$lib/types";
  import { getSynergyPercentageDetailsSum } from "$lib/utils/buffs";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import BuffDetailTooltip from "./tooltips/BuffDetailTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";

  interface Props {
    enc: EncounterState;
    player: Entity;
    groupedSynergies: Map<string, Map<number, StatusEffect>>;
    percentage: number;
  }

  let { enc, player, groupedSynergies, percentage }: Props = $props();

  let entityState = $derived(new EntityState(player, enc));

  let synergyPercentageDetails: Array<BuffDetails> = $derived(
    getSynergyPercentageDetailsSum(groupedSynergies, entityState)
  );

  const tweenedValue = new Tween(enc.live ? 0 : percentage, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(percentage ?? 0);
  });

  let alpha = $derived(enc.live && !settings.app.meter.showClassColors ? 0 : 0.6);
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
{#if groupedSynergies.size > 0}
  {#each synergyPercentageDetails as synergy}
    <td class="px-1 text-center text-sm text-neutral-200">
      {#if synergy.percentage}
        <BuffDetailTooltip buffDetails={synergy} />
      {/if}
    </td>
  {/each}
{/if}
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {tweenedValue.current}%"
></td>
