<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import type { Entity } from "$lib/types";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import ArkPassiveTooltip from "./tooltips/ArkPassiveTooltip.svelte";
  import { rgbLinearShadeAdjust } from "$lib/utils";
  import { damageValue } from "./Snippets.svelte";

  interface Props {
    enc: EncounterState;
    boss: Entity;
    width: number;
    index: number;
  }

  let { enc, boss, width, index }: Props = $props();
  let entityState = $derived(new EntityState(boss, enc));

  const tweenedValue = new Tween(enc.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });

  let color = "#164e63";
</script>

<td colspan="2" class="px-2">
  <div class="flex truncate">
    <ArkPassiveTooltip state={entityState} />
  </div>
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={entityState.damageDealt.toLocaleString()}>
    {@render damageValue(entityState.damageDealtString)}
  </QuickTooltip>
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={entityState.dps.toLocaleString()}>
    {@render damageValue(entityState.dpsString)}
  </QuickTooltip>
</td>
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: {index % 2 === 1 && settings.app.general.splitLines
    ? rgbLinearShadeAdjust(color, -0.2, 0.6)
    : `rgb(from ${color} r g b / 0.6)`}; width: {tweenedValue.current}%"
></td>
