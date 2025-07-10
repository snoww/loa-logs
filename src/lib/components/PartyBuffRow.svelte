<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import type { BuffDetails, Entity } from "$lib/types";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import BuffDetailTooltip from "./tooltips/BuffDetailTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";
  import { settings } from "$lib/stores.svelte.js";

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

  let alpha = $derived(enc.live && !settings.app.meter.showClassColors ? 0 : 0.6);

  $effect(() => {
    tweenedValue.set(percentage ?? 0);
  });
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
{#if playerBuffs.length > 0}
  {#each playerBuffs as buff (buff.id)}
    <td class="text-sm px-1 text-center text-neutral-200">
      {#if buff.percentage}
        <BuffDetailTooltip buffDetails={buff} />
      {/if}
    </td>
  {/each}
{/if}
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {tweenedValue.current}%"
></td>
