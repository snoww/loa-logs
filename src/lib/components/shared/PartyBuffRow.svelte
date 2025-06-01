<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { EntityState } from "$lib/entity.svelte";
  import type { BuffDetails, Entity } from "$lib/types";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import ClassTooltip from "../tooltips/ClassTooltip.svelte";
  import QuickTooltip from "../QuickTooltip.svelte";
  import BuffDetailTooltip from "../tooltips/BuffDetailTooltip.svelte";

  interface Props {
    player: Entity;
    enc: EncounterState;
    playerBuffs: Array<BuffDetails>;
    percentage: number;
  }

  let { player, enc, playerBuffs, percentage }: Props = $props();
  let entityState = $derived(new EntityState(player, enc));

  const tweenedValue = new Tween(enc.live ? 0 : percentage, {
    duration: 400,
    easing: cubicOut
  });

  let alpha = $derived(enc.live && !enc.curSettings.showClassColors ? 0 : 0.6);

  $effect(() => {
    tweenedValue.set(percentage ?? 0);
  });
</script>

<td class="pl-1">
  <ClassTooltip entity={player} />
</td>
<td colspan="2">
  <div class="flex truncate">
    <QuickTooltip tooltip={entityState.name}>
      {entityState.name}
    </QuickTooltip>
  </div>
</td>
{#if playerBuffs.length > 0}
  {#each playerBuffs as buff (buff.id)}
    <td class="text-3xs px-1 text-center">
      {#if buff.percentage}
        <BuffDetailTooltip buffDetails={buff} />
      {/if}
    </td>
  {/each}
{/if}
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {percentage}%"
></td>
