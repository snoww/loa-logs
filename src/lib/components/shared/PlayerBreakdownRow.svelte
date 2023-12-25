<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateTripodTooltip, tooltip } from "$lib/utils/tooltip";

    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let anySupportBuff: boolean;
    export let anySupportBrand: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let width: number;
    export let duration: number;

    export let meterSettings: any;
    export let shadow: boolean = false;
    export let index: number;

    let critPercentage = "0.0";
    let critDmgPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";
    $: {
        if (skill.hits !== 0) {
            critDmgPercentage = round((skill.critDamage / skill.totalDamage) * 100);
            critPercentage = round((skill.crits / skill.hits) * 100);
            if (
                meterSettings.positionalDmgPercent &&
                (skill.frontAttackDamage > 0 || skill.backAttackDamage > 0)
            ) {
                faPercentage = round((skill.frontAttackDamage / skill.totalDamage) * 100);
                baPercentage = round((skill.backAttackDamage / skill.totalDamage) * 100);
            } else {
                faPercentage = round((skill.frontAttacks / skill.hits) * 100);
                baPercentage = round((skill.backAttacks / skill.hits) * 100);
            }
        }
    }
</script>

<td class="pl-1">
    <img
        class="h-5 w-5"
        src={$skillIcon.path + getSkillIcon(skill.icon)}
        alt={skill.name}
        use:tooltip={{ content: skill.name }} />
</td>
<td class="-left-px" colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: generateTripodTooltip(skill) }}>
            {skill.name}
        </span>
    </div>
</td>
{#if meterSettings.breakdown.damage}
    <td class="px-1 text-center" use:tooltip={{ content: skill.totalDamage.toLocaleString() }}>
        {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.dps}
    <td class="px-1 text-center">
        {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.damagePercent}
    <td class="px-1 text-center">
        {round((skill.totalDamage / playerDamageDealt) * 100)}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.breakdown.critRate}
    <td class="px-1 text-center">
        {critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.breakdown.critDmg}
    <td class="px-1 text-center">
        {critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if hasFrontAttacks && meterSettings.breakdown.frontAtk}
    <td class="px-1 text-center">
        {faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if hasBackAttacks && meterSettings.breakdown.backAtk}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBuff && meterSettings.breakdown.percentBuffBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.buffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if anySupportBrand && meterSettings.breakdown.percentBrand}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.debuffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if meterSettings.breakdown.avgDamage}
    <td class="px-1 text-center">
        {abbreviateNumberSplit(skill.totalDamage / skill.hits)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.totalDamage / skill.hits)[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.avgDamage}
    <td class="px-1 text-center">
        {abbreviateNumberSplit(skill.totalDamage / skill.casts)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.totalDamage / skill.casts)[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.maxDamage}
    <td class="px-1 text-center">
        {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.casts}
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
{/if}
{#if meterSettings.breakdown.cpm}
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
{/if}
{#if meterSettings.breakdown.hits}
    <td
        class="px-1 text-center"
        use:tooltip={{
            content: `<div class="py-1">${
                skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")
            }</div>`
        }}>
        {abbreviateNumberSplit(skill.hits)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.hits)[1]}</span>
    </td>
{/if}
{#if meterSettings.breakdown.hpm}
    <td class="px-1 text-center">
        {#if skill.hits === 0}
            <div class="">0</div>
        {:else}
            <div
                use:tooltip={{
                    content: `<div class="py-1">${
                        skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")
                    }</div>`
                }}>
                {round(skill.hits / (duration / 1000 / 60))}
            </div>
        {/if}
    </td>
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {width}%" />
