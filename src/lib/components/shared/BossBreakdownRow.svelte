<script lang="ts">
  import type { EntityState } from "$lib/entity.svelte";
  import { SkillState } from "$lib/skill.svelte";
  import type { Skill } from "$lib/types";
  import { abbreviateNumberSplit, customRound } from "$lib/utils/numbers";
  import { getSkillIcon, rgbLinearShadeAdjust } from "$lib/utils/strings";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "../QuickTooltip.svelte";
  import SkillTooltip from "../tooltips/SkillTooltip.svelte";
  import { settings } from "$lib/stores.svelte";

  interface Props {
    skill: Skill;
    entityState: EntityState;
    width: number;
    index: number;
  }

  let { entityState, skill, width, index }: Props = $props();

  let skillState = $derived(new SkillState(skill, entityState));

  let color = "#164e63";

  const tweenedValue = new Tween(entityState.encounter.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

{#snippet skillTooltip()}
  <SkillTooltip {skill} />
{/snippet}
<!-- Render value + units -->
{#snippet damageValue(val: [number, string])}
  {val[0]}<span class="text-3xs text-gray-300">{val[1]}</span>
{/snippet}
<!-- Render value + percent -->
{#snippet percentValue(val: string | number)}
  {val}<span class="text-3xs text-gray-300">%</span>
{/snippet}

<td class="pl-1">
  <QuickTooltip tooltip={skill.name}>
    <img class="size-5" src={getSkillIcon(skill.icon)} alt={skill.name} />
  </QuickTooltip>
</td>

<td class="-left-px" colspan="2">
  <div class="flex truncate">
    <QuickTooltip tooltip={skillTooltip} class="truncate">
      {skill.name}
    </QuickTooltip>
  </div>
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={skill.totalDamage.toLocaleString()}>
    {@render damageValue(skillState.skillDamageString)}
  </QuickTooltip>
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={skillState.skillDps.toLocaleString()}>
    {@render damageValue(skillState.skillDpsString)}
  </QuickTooltip>
</td>
<td class="px-1 text-center">
  {@render percentValue(customRound((skill.totalDamage / entityState.damageDealt) * 100))}
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}>
    {@render damageValue(abbreviateNumberSplit(skill.casts))}
  </QuickTooltip>
</td>
<td class="px-1 text-center">
  <QuickTooltip tooltip={skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}>
    {customRound(skill.casts / (entityState.encounter.duration / 1000 / 60))}
  </QuickTooltip>
</td>
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: {index % 2 === 1 && settings.app.general.splitLines
    ? rgbLinearShadeAdjust(color, -0.2, 0.6)
    : `rgb(from ${color} r g b / 0.6)`}; width: {tweenedValue.current}%"
></td>
