<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { EntityState } from "$lib/entity.svelte";
  import type { Entity } from "$lib/types";
  import { getClassIcon } from "$lib/utils";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";

  let { entity, enc, width }: { entity: Entity; enc: EncounterState; width: number } = $props();

  let state = new EntityState(entity, enc);

  let tweenedValue = new Tween(width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

{#snippet damageValue(val: [number, string])}
  {val[0]}<span class="text-xxs text-gray-300">{val[1]}</span>
{/snippet}
{#snippet percentValue(val: string | number)}
  {val}<span class="text-xxs text-gray-300">%</span>
{/snippet}

<!-- name -->
<div class="flex items-center justify-center gap-1 truncate">
  <img src={getClassIcon(state.entity.classId)} class="size-5" alt={state.entity.class} />
  <p class="truncate">{entity.name}</p>
</div>
<!-- stats -->
<div class="flex h-6 w-full items-center justify-between px-2">
  <div>
    {@render damageValue(state.dpsString)}
  </div>
  <div>
    {@render percentValue(state.damagePercentage)}
  </div>
</div>

<!-- dmg% bar -->
<div
  class="absolute bottom-0 left-0 -z-10 h-6"
  style="background-color: rgb(from {state.color} r g b / {0.6}); width: {tweenedValue.current}%"
></div>
<!-- background bar -->
<div
  class="absolute bottom-0 left-0 -z-20 h-6 w-full"
  style="background-color: rgb(from {state.color} r g b / {0.3})"
></div>
