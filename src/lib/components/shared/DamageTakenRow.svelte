<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { EntityState } from "$lib/entity.svelte";
  import type { Entity } from "$lib/types";
  import { HexToRgba } from "$lib/utils/colors";
  import { classIconCache } from "$lib/utils/settings";
  import { takingScreenshot } from "$lib/utils/stores";
  import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";

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

  let alpha = $derived(enc.live && !enc.curSettings.showClassColors ? 0 : 0.6);
</script>

<td class="pl-1">
  <img
    class="table-cell size-5"
    src={$classIconCache[player.classId]}
    alt={player.class}
    use:tooltip={{ content: generateClassTooltip(player) }}
  />
</td>
<td colspan="2">
  <div class="truncate">
    <span use:tooltip={{ content: entityState.name }}>
      {entityState.name}
    </span>
  </div>
</td>
<td class="pl-1 pr-2 text-right">
  <span use:tooltip={{ content: player.damageStats.damageTaken.toLocaleString() }}>
    {entityState.damageTakenString[0]}<span class="text-3xs text-gray-300">{entityState.damageTakenString[1]}</span>
  </span>
</td>
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  class:shadow-md={!$takingScreenshot}
  style="background-color: {HexToRgba(entityState.color, alpha)}; width: {tweenedValue.current}%"
></td>
