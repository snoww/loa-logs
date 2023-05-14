<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { getSkillIcon } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";

    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let anySupportBuff: boolean;
    export let anySupportBrand: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;
    export let duration: number;

    let critPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    if (skill.hits !== 0) {
        critPercentage = round(skill.crits / skill.hits * 100);
        faPercentage = round(skill.frontAttacks / skill.hits * 100) ;
        baPercentage = round(skill.backAttacks / skill.hits * 100);
    }

</script>

<tr class="h-7 px-2 py-1 text-3xs">
    <td class="pl-1">
        <img class="h-5 w-5" src={$skillIcon.path + getSkillIcon(skill.icon)} alt={skill.name} use:tooltip={{content: skill.name}}/>
    </td>
    <td class="-left-px" colspan="2">
        <div class="truncate">
            {skill.name}
        </div>
    </td>
    {#if $settings.logs.breakdown.damage}
    <td class="px-1 text-center">
        {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
    </td>
    {/if}
    {#if $settings.logs.breakdown.dps}
    <td class="px-1 text-center">
        {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
    </td>
    {/if}
    {#if $settings.logs.breakdown.damagePercent}
    <td class="px-1 text-center">
        {round(skill.totalDamage / playerDamageDealt * 100)}<span class="text-xs text-gray-300">%</span>
    </td>
    {/if}
    {#if $settings.logs.breakdown.critRate}
    <td class="px-1 text-center">
        {critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {/if}
    {#if hasFrontAttacks && $settings.logs.breakdown.frontAtk}
    <td class="px-1 text-center">
        {faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {/if}
    {#if hasBackAttacks && $settings.logs.breakdown.backAtk}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {/if}
    {#if anySupportBuff && $settings.logs.breakdown.percentBuffBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
        {round(skill.buffedBySupport / skill.totalDamage * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
        0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
    {/if}
    {#if anySupportBrand && $settings.logs.breakdown.percentBrand}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
        {round(skill.debuffedBySupport / skill.totalDamage * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
        0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
    {/if}
    {#if $settings.logs.breakdown.avgDamage}
    <td class="px-1 text-center">
        {abbreviateNumberSplit(skill.totalDamage / skill.hits)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.totalDamage / skill.hits)[1]}</span>
    </td>
    {/if}
    {#if $settings.logs.breakdown.maxDamage}
    <td class="px-1 text-center">
        {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
    </td>
    {/if}
    {#if $settings.logs.breakdown.casts}
    <td class="px-1 text-center">
        {skill.casts.toLocaleString()}
    </td>
    {/if}
    {#if $settings.logs.breakdown.cpm}
    <td class="px-1 text-center">
        <div use:tooltip={{content: `<div class="py-1">${skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}</div>`}}>
            {round(skill.casts / (duration / 1000 / 60))}
        </div>
    </td>
    {/if}
    {#if $settings.logs.breakdown.hits}
    <td class="px-1 text-center">
        {skill.hits.toLocaleString()}
    </td>
    {/if}
    {#if $settings.logs.breakdown.hpm}
    <td class="px-1 text-center">
        {#if skill.hits === 0}
        <div class="">
            0
        </div>
        {:else}
        <div use:tooltip={{content: `<div class="py-1">${skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")}</div>`}}>
            {round(skill.hits / (duration / 1000 / 60))}
        </div>
        {/if}
    </td>
    {/if}
    <div class="absolute left-0 h-7 px-2 py-1 -z-10" class:shadow-md={!$takingScreenshot}
        style="background-color: {HexToRgba(color, 0.6)}; width: {damagePercentage}%"
    ></div>
</tr>
