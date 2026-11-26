<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";

  import PersonalBestIndicator from "#lib/components/PersonalBestIndicator.svelte";
  import { damageValue, percentValue } from "#lib/components/Snippets.svelte";
  import type { EncounterState } from "#lib/encounter.svelte.ts";
  import { EntityState } from "#lib/entity.svelte.ts";
  import { settings } from "#lib/stores.svelte.ts";
  import type { Entity } from "#lib/types.ts";
  import { customRound, getClassIcon } from "#lib/utils.ts";

  import { previewNdpsDelta } from "./preview";

  let { entity, enc, width }: { entity: Entity; enc: EncounterState; width: number } = $props();

  let state = new EntityState(entity, enc);
  $effect(() => {
    state.entity = entity;
    state.encounter = enc;
  });

  let tweenedValue = new Tween(width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

{#snippet buffSummary()}
  <span class="">
    {customRound(
      (state.entity.damageStats.buffedBySupport / state.damageDealtWithoutSpecialOrHa) * 100,
      0
    )}/{customRound(
      (state.entity.damageStats.debuffedBySupport / state.damageDealtWithoutSpecialOrHa) * 100,
      0
    )}/{customRound((state.entity.damageStats.buffedByIdentity / state.damageDealtWithoutSpecialOrHa) * 100, 0)}
  </span>
{/snippet}

<!-- name -->
<div class="flex items-center justify-center gap-1">
  <img src={getClassIcon(state.entity.classId)} class="size-5 shrink-0" alt={state.entity.class} />
  <p class="truncate">{state.name}</p>
  {#if settings.app.meter.personalBest && entity.name === enc.localPlayer && (enc.personalBest || settings.app.general.miniEdit)}
    <PersonalBestIndicator
      {enc}
      entityState={state}
      compact
      previewDelta={settings.app.general.miniEdit ? previewNdpsDelta : undefined}
    />
  {/if}
</div>
<!-- stats -->
<div class="flex h-6 w-full items-center justify-between truncate px-2">
  <div>
    {@render damageValue(state.dpsString)}
  </div>
  {#if settings.app.mini.info === "damage"}
    <div>
      {@render percentValue(state.damagePercentage)}
    </div>
  {:else if settings.app.mini.info === "buff"}
    <div class="truncate">
      {@render buffSummary()}
    </div>
  {/if}
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
