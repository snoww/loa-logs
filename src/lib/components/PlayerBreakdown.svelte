<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { IconExternalLink } from "$lib/icons";
  import { settings } from "$lib/stores.svelte.js";
  import { EntityType, type Entity } from "$lib/types";
  import { customRound, isNameValid, rgbLinearShadeAdjust, UWUOWO_URL } from "$lib/utils";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { flip } from "svelte/animate";
  import PlayerBreakdownHeader from "./PlayerBreakdownHeader.svelte";
  import PlayerBreakdownRow from "./PlayerBreakdownRow.svelte";
  import { badTooltip, fadTooltip } from "./Snippets.svelte";
  import ArkPassiveTooltip from "./tooltips/ArkPassiveTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";

  interface Props {
    handleRightClick: () => void;
    entity: Entity;
    enc: EncounterState;
  }

  let { entity, enc, handleRightClick }: Props = $props();
  let entityState = $derived(new EntityState(entity, enc));
</script>

<thead class="z-30 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
  <tr class="bg-neutral-950/80">
    <PlayerBreakdownHeader {entityState} {handleRightClick} />
  </tr>
</thead>

<tbody class="relative z-10">
  {#if entity.entityType !== EntityType.ESTHER}
    <tr class="text-xxs h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
      <td class="pl-1">
        <ClassTooltip {entity} />
      </td>
      <td colspan="2">
        <div class="flex gap-1">
          <div class="truncate">
            <ArkPassiveTooltip state={entityState} />
          </div>
          {#if isNameValid(entityState.entity.name) && entityState.entity.entityType === EntityType.PLAYER}
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
      {#if enc.curSettings.breakdown.damage}
        <td class="px-1 text-center">
          <QuickTooltip tooltip={entity.damageStats.damageDealt.toLocaleString()}>
            {entityState.damageDealtString[0]}<span class="text-xxs text-gray-300"
              >{entityState.damageDealtString[1]}</span
            >
          </QuickTooltip>
        </td>
      {/if}
      {#if enc.curSettings.breakdown.dps}
        <td class="px-1 text-center">
          <QuickTooltip tooltip={entity.damageStats.dps.toLocaleString()}>
            {entityState.dpsString[0]}<span class="text-xxs text-gray-300">{entityState.dpsString[1]}</span>
          </QuickTooltip>
        </td>
      {/if}
      {#if enc.curSettings.breakdown.damagePercent}
        <td class="px-1 text-center">
          {entityState.damagePercentage}<span class="text-xs text-gray-300">%</span>
        </td>
      {/if}
      {#if enc.curSettings.breakdown.critRate}
        <td class="px-1 text-center">
          {entityState.critPercentage}<span class="text-xxs text-gray-300">%</span>
        </td>
      {/if}
      {#if !enc.live && settings.app.logs.breakdown.adjustedCritRate}
        <td class="px-1 text-center"> - </td>
      {/if}
      {#if enc.curSettings.breakdown.critDmg}
        <td class="px-1 text-center">
          {entityState.critDmgPercentage}<span class="text-xxs text-gray-300">%</span>
        </td>
      {/if}
      {#if entityState.anyFrontAttacks && enc.curSettings.breakdown.frontAtk && !enc.curSettings.positionalDmgPercent}
        <td class="px-1 text-center">
          {entityState.faPercentage}<span class="text-xxs text-gray-300">%</span>
        </td>
      {/if}
      {#if entityState.anyFrontAttacks && enc.curSettings.breakdown.frontAtk && enc.curSettings.positionalDmgPercent}
        <td class="px-1 text-center">
          <QuickTooltip tooltip={fadTooltip} tooltipProps={entityState}>
            {entityState.fadPercentage}<span class="text-xxs text-gray-300">%</span>
          </QuickTooltip>
        </td>
      {/if}
      {#if entityState.anyBackAttacks && enc.curSettings.breakdown.backAtk && !enc.curSettings.positionalDmgPercent}
        <td class="px-1 text-center">
          {entityState.baPercentage}<span class="text-xxs text-gray-300">%</span>
        </td>
      {/if}
      {#if entityState.anyBackAttacks && enc.curSettings.breakdown.backAtk && enc.curSettings.positionalDmgPercent}
        <td class="px-1 text-center">
          <QuickTooltip tooltip={badTooltip} tooltipProps={entityState}>
            {entityState.badPercentage}<span class="text-xxs text-gray-300">%</span>
          </QuickTooltip>
        </td>
      {/if}
      {#if entityState.anySupportBuff && enc.curSettings.breakdown.percentBuffBySup}
        <td class="px-1 text-center">
          {customRound((entity.damageStats.buffedBySupport / entityState.damageDealtWithoutSpecialOrHa) * 100)}<span
            class="text-xxs text-gray-300">%</span
          >
        </td>
      {/if}
      {#if entityState.anySupportBrand && enc.curSettings.breakdown.percentBrand}
        <td class="px-1 text-center">
          {customRound((entity.damageStats.debuffedBySupport / entityState.damageDealtWithoutSpecialOrHa) * 100)}<span
            class="text-xxs text-gray-300">%</span
          >
        </td>
      {/if}
      {#if entityState.anySupportIdentity && enc.curSettings.breakdown.percentIdentityBySup}
        <td class="px-1 text-center">
          {customRound((entity.damageStats.buffedByIdentity / entityState.damageDealtWithoutSpecialOrHa) * 100)}<span
            class="text-xxs text-gray-300">%</span
          >
        </td>
      {/if}
      {#if entityState.anySupportHat && enc.curSettings.breakdown.percentHatBySup}
        <td class="px-1 text-center">
          {customRound(((entity.damageStats.buffedByHat ?? 0) / entityState.damageDealtWithoutSpecial) * 100)}<span
            class="text-xxs text-gray-300">%</span
          >
        </td>
      {/if}
      {#if enc.curSettings.breakdown.avgDamage}
        <td class="px-1 text-center"> - </td>
        <td class="px-1 text-center"> - </td>
      {/if}
      {#if enc.curSettings.breakdown.maxDamage}
        <td class="px-1 text-center"> - </td>
        <td class="px-1 text-center" class:hidden={enc.live}> - </td>
      {/if}
      {#if enc.curSettings.breakdown.casts}
        <td class="px-1 text-center">
          {entity.skillStats.casts}
        </td>
      {/if}
      {#if enc.curSettings.breakdown.cpm}
        <td class="px-1 text-center">
          {customRound(entity.skillStats.casts / (entityState.encounter.duration / 1000 / 60))}
        </td>
      {/if}
      {#if enc.curSettings.breakdown.hits}
        <td class="px-1 text-center">
          {entity.skillStats.hits}
        </td>
      {/if}
      {#if enc.curSettings.breakdown.hpm}
        <td class="px-1 text-center">
          {#if entity.skillStats.hits === 0}
            0
          {:else}
            {customRound(entity.skillStats.hits / (entityState.encounter.duration / 1000 / 60))}
          {/if}
        </td>
      {/if}
      {#if entityState.anyCooldownRatio}
        <td class="px-1 text-center">-</td>
      {/if}
      <td
        class="absolute left-0 -z-10 h-7 px-2 py-1"
        style="background-color: {settings.app.general.splitLines
          ? rgbLinearShadeAdjust(entityState.color, -0.2, 0.6)
          : `rgb(from ${entityState.color} r g b / 0.6)`}; width: 100%"
      ></td>
    </tr>
  {/if}
  {#each entityState.skills as skill, i (skill.id)}
    <tr
      animate:flip={{ duration: 200 }}
      class="text-xxs h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
    >
      <PlayerBreakdownRow {skill} {entityState} width={entityState.skillDamagePercentages[i]!} index={i} />
    </tr>
  {/each}
</tbody>
