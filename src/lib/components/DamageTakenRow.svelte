<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import type { Entity } from "$lib/types";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";
  import { settings } from "$lib/stores.svelte.js";

  interface Props {
    enc: EncounterState;
    player: Entity;
    width: number;
  }

  let { enc, player, width }: Props = $props();
  let entityState = $derived(new EntityState(player, enc));

  const tweenedValue = new Tween(enc.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });

  let alpha = $derived(enc.live && !settings.app.meter.showClassColors ? 0 : 0.6);
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
<td class="pl-1 pr-2 text-right">
  <QuickTooltip tooltip={player.damageStats.damageTaken.toLocaleString()}>
    {entityState.damageTakenString[0]}<span class="text-xxs text-gray-300">{entityState.damageTakenString[1]}</span>
  </QuickTooltip>
</td>
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {tweenedValue.current}%"
></td>
