<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { estherNameToIcon } from "$lib/constants/esthers";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { IconExternalLink } from "$lib/icons";
  import { settings } from "$lib/stores.svelte.js";
  import { EntityType, type Entity } from "$lib/types";
  import { cubicOut } from "svelte/easing";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Tween } from "svelte/motion";
  import ArkPassiveTooltip from "./tooltips/ArkPassiveTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";
  import { logColumns } from "./DamageMeterColumns.svelte";
  import { getClassIcon, isNameValid, UWUOWO_URL } from "$lib/utils";

  interface Props {
    enc: EncounterState;
    entity: Entity;
    width: number;
    shadow?: boolean;
  }

  let { enc, entity, width, shadow = false }: Props = $props();

  let entityState = $derived(new EntityState(entity, enc));
  let hovering = $state(false);

  let tweenedValue = new Tween(enc.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });
  $effect(() => {
    tweenedValue.set(width ?? 0);
  });

  let alpha = $derived(enc.live && !settings.app.meter.showClassColors ? 0 : 0.6);
</script>

<td class="pl-1">
  {#if settings.app.general.showEsther && entity.entityType === EntityType.ESTHER}
    <QuickTooltip tooltip={entityState.name}>
      <img class="table-cell size-5" src={getClassIcon(estherNameToIcon[entityState.name])} alt={entityState.name} />
    </QuickTooltip>
  {:else}
    <ClassTooltip {entity} />
  {/if}
</td>

<td colspan="2" onmouseenter={() => (hovering = true)} onmouseleave={() => (hovering = false)}>
  <div class="flex gap-1">
    <div class="truncate">
      <ArkPassiveTooltip state={entityState} />
    </div>
    {#if (enc.live && settings.app.meter.profileShortcut) || (!enc.live && isNameValid(entityState.entity.name) && hovering && entityState.entity.entityType === EntityType.PLAYER)}
      <button
        class="shrink-0"
        onclick={(e) => {
          e.stopPropagation();
          openUrl(UWUOWO_URL + "/character/" + enc.region + "/" + entityState.entity.name);
        }}
      >
        <IconExternalLink class="size-3" />
      </button>
    {/if}
  </div>
</td>

{#each logColumns as columnDef}
  {#if columnDef.show(enc)}
    <td class="cursor-default px-1 text-center">
      {#snippet tooltip()}
        {#if columnDef.valueTooltip}
          {@render columnDef.valueTooltip(entityState)}
        {/if}
      {/snippet}

      <QuickTooltip tooltip={columnDef.valueTooltip ? tooltip : null}>
        {@render columnDef.value(entityState)}
      </QuickTooltip>
    </td>
  {/if}
{/each}

<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  class:shadow-md={shadow}
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {tweenedValue.current}%"
></td>
