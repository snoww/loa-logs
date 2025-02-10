<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    interface Props {
        skill: Skill;
        abbreviatedSkillDamage: (string | number)[];
        skillDps: (string | number)[];
        width: number;
        shadow?: boolean;
        index: number;
        duration: number;
        totalDamageDealt: number;
        tween: boolean;
    }

    let {
        skill,
        abbreviatedSkillDamage,
        skillDps,
        width,
        shadow = false,
        index,
        duration,
        totalDamageDealt,
        tween
    }: Props = $props();

    let color = "#164e63";

    const tweenedValue = new Tween(0, {
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
    {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
</td>
<td class="px-1 text-center">
    {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
</td>
<td class="px-1 text-center">
    {round((skill.totalDamage / totalDamageDealt) * 100)}<span class="text-xs text-gray-300">%</span>
</td>
<td
    class="px-1 text-center"
    use:tooltip={{
        content: `<div class="py-1">${
            skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")
        }</div>`
    }}>
    {abbreviateNumberSplit(skill.casts)[0]}<span class="text-3xs text-gray-300"
        >{abbreviateNumberSplit(skill.casts)[1]}</span>
</td>
<td class="px-1 text-center">
    <div
        use:tooltip={{
            content: `<div class="py-1">${
                skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")
            }</div>`
        }}>
        {round(skill.casts / (duration / 1000 / 60))}
    </div>
</td>
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {tween ? tweenedValue.current : width}%"></td>
