<script lang="ts">
  import type { EntityState } from "$lib/entity.svelte.js";
  import { SkillState } from "$lib/skill.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import type { Skill } from "$lib/types";
  import { getSkillIcon, rgbLinearShadeAdjust } from "$lib/utils";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import { skillTooltip } from "./Snippets.svelte";
  import { playerBreakdownColumns } from "./PlayerBreakdownColumns.svelte";

  interface Props {
    skill: Skill;
    entityState: EntityState;
    width: number;
    shadow?: boolean;
    index: number;
  }

  let { skill, entityState, width, shadow = false, index }: Props = $props();

  let skillState = $derived(new SkillState(skill, entityState));
  let tweenedValue = new Tween(entityState.encounter.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });
  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

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

{#each playerBreakdownColumns as columnDef}
  {#if columnDef.show(entityState)}
    <td class="cursor-default px-1 text-center">
      {#snippet tooltip()}
        {#if columnDef.valueTooltip}
          {@render columnDef.valueTooltip(skillState)}
        {/if}
      {/snippet}

      <QuickTooltip tooltip={columnDef.valueTooltip ? tooltip : null}>
        {@render columnDef.value(skillState)}
      </QuickTooltip>
    </td>
  {/if}
{/each}

<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  class:shadow-md={shadow}
  style="background-color: {index % 2 === 1 && settings.app.general.splitLines
    ? rgbLinearShadeAdjust(entityState.color, -0.2, 0.6)
    : `rgb(from ${entityState.color} r g b / 0.6)`}; width: {tweenedValue.current}%"
></td>
