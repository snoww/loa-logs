<script lang="ts">
  import type { EntityState } from "$lib/entity.svelte";
  import { SkillState } from "$lib/skill.svelte";
  import type { Skill } from "$lib/types";
  import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
  import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
  import { settings } from "$lib/utils/settings";
  import { takingScreenshot } from "$lib/utils/stores";
  import { tooltip } from "$lib/utils/tooltip";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";

  interface Props {
    skill: Skill;
    entityState: EntityState;
    width: number;
    index: number;
  }

  let { entityState, skill, width, index }: Props = $props();

  let skillState = $derived(new SkillState(skill, entityState));

  let color = "#164e63";

  const tweenedValue = new Tween(entityState.enc.live ? 0 : width, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(width ?? 0);
  });
</script>

<td class="px-2" colspan="2">
  <div class="truncate">
    <span use:tooltip={{ content: skill.name }}>
      {skill.name}
    </span>
  </div>
</td>
<td class="px-1 text-center" use:tooltip={{ content: skill.totalDamage.toLocaleString() }}>
  {skillState.skillDamageString[0]}<span class="text-3xs text-gray-300">{skillState.skillDamageString[1]}</span>
</td>
<td class="px-1 text-center">
  {skillState.skillDpsString[0]}<span class="text-3xs text-gray-300">{skillState.skillDpsString[1]}</span>
</td>
<td class="px-1 text-center">
  {round((skill.totalDamage / entityState.damageDealt) * 100)}<span class="text-xs text-gray-300">%</span>
</td>
<td
  class="px-1 text-center"
  use:tooltip={{
    content: `<div class="py-1">${skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}</div>`
  }}
>
  {abbreviateNumberSplit(skill.casts)[0]}<span class="text-3xs text-gray-300"
    >{abbreviateNumberSplit(skill.casts)[1]}</span
  >
</td>
<td class="px-1 text-center">
  <div
    use:tooltip={{
      content: `<div class="py-1">${skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}</div>`
    }}
  >
    {round(skill.casts / (entityState.enc.duration / 1000 / 60))}
  </div>
</td>
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  class:shadow-md={!takingScreenshot}
  style="background-color: {index % 2 === 1 && $settings.general.splitLines
    ? RGBLinearShade(HexToRgba(color, 0.6))
    : HexToRgba(color, 0.6)}; width: {tweenedValue.current}%"
></td>
