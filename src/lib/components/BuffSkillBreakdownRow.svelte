<script lang="ts">
  import type { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import type { BuffDetails, Skill, StatusEffect } from "$lib/types";
  import { getSkillIcon, rgbLinearShadeAdjust } from "$lib/utils";
  import { getSynergyPercentageDetails, hyperAwakeningIds } from "$lib/utils/buffs";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import { skillTooltip } from "./Snippets.svelte";
  import BuffDetailTooltip from "./tooltips/BuffDetailTooltip.svelte";

  interface Props {
    skill: Skill;
    entityState: EntityState;
    groupedSynergies: Map<string, Map<number, StatusEffect>>;
    width: number;
    index: number;
  }

  let { skill, entityState, groupedSynergies, width, index }: Props = $props();

  let synergyPercentageDetails: Array<BuffDetails> = $derived(getSynergyPercentageDetails(groupedSynergies, skill));
  let isHyperAwakening = skill.isHyperAwakening || hyperAwakeningIds.has(skill.id);

  const tweenedValue = new Tween(entityState.encounter.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });
  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

<tr class="text-xxs h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
  <td class="pl-1">
    <QuickTooltip tooltip={skill.name}>
      <img class="size-5" src={getSkillIcon(skill.icon)} alt={skill.name} />
    </QuickTooltip>
  </td>
  <td class="-left-px" colspan="2">
    <div class="flex truncate">
      <QuickTooltip tooltip={skillTooltip} tooltipProps={skill} class="truncate">
        {skill.name}
      </QuickTooltip>
    </div>
  </td>
  {#if groupedSynergies.size > 0}
    {#each synergyPercentageDetails as synergy (synergy.id)}
      <td class="px-1 text-center">
        {#if synergy.percentage}
          <BuffDetailTooltip buffDetails={synergy} />
        {:else if skill.special || isHyperAwakening}
          -
        {:else}
          0<span class="text-xxs text-neutral-300">%</span>
        {/if}
      </td>
    {/each}
  {/if}
  <td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    style="background-color: {index % 2 === 1 && settings.app.general.splitLines
      ? rgbLinearShadeAdjust(entityState.color, -0.2, 0.6)
      : `rgb(from ${entityState.color} r g b / 0.6)`}; width: {tweenedValue.current}%"
  ></td>
</tr>
