<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { getSkillIcon } from "$lib/utils/strings";
    import { Tooltip } from "flowbite-svelte";

    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;
    export let duration: number;

    let critPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    if (skill.hits !== 0) {
        critPercentage = (skill.crits / skill.hits * 100).toFixed(1);
        faPercentage = (skill.frontAttacks / skill.hits * 100).toFixed(1) ;
        baPercentage = (skill.backAttacks / skill.hits * 100).toFixed(1);
    }

</script>


<td class="px-1 relative z-10">
    <div class="flex space-x-1 items-center">
        <img class="h-5 w-5" src={$skillIcon.path + getSkillIcon(skill.icon)} alt={skill.name} />
        <div class="truncate">
            {skill.name}
        </div>
    </div>
</td>
{#if $settings.logs.breakdown.damage}
<td class="px-1 text-center relative z-10">
    {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
</td>
{/if}
{#if $settings.logs.breakdown.dps}
<td class="px-1 text-center relative z-10">
    {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
</td>
{/if}
{#if $settings.logs.breakdown.damagePercent}
<td class="px-1 text-center relative z-10">
    {(skill.totalDamage / playerDamageDealt * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.breakdown.critRate}
<td class="px-1 text-center relative z-10">
    {critPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if hasFrontAttacks && $settings.logs.breakdown.frontAtk}
<td class="px-1 text-center relative z-10">
    {faPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if hasBackAttacks && $settings.logs.breakdown.backAtk}
<td class="px-1 text-center relative z-10">
    {baPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.breakdown.avgDamage}
<td class="px-1 text-center relative z-10">
    {abbreviateNumberSplit(skill.totalDamage / skill.hits)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.totalDamage / skill.hits)[1]}</span>
</td>
{/if}
{#if $settings.logs.breakdown.maxDamage}
<td class="px-1 text-center relative z-10">
    {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
</td>
{/if}
{#if $settings.logs.breakdown.casts}
<td class="px-1 text-center relative z-10">
    <div class="">
        {(skill.casts / (duration / 1000 / 60)).toFixed(1)}
    </div>
    <Tooltip placement="top" class="bg-zinc-900 text-gray-300 text-xs" style="custom">
        <div class="truncate">
            {skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")}
        </div>
    </Tooltip>
</td>
{/if}
{#if $settings.logs.breakdown.hits}
<td class="px-1 text-center relative z-10">
    {#if skill.hits === 0}
    <div class="">
        0
    </div>
    {:else}
    <div class="">
        {(skill.hits / (duration / 1000 / 60)).toFixed(1)}
    </div>
    {/if}
    <Tooltip placement="top" class="bg-zinc-900 text-gray-300 text-xs" style="custom">
        <div class="truncate">
            {skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")}
        </div>
    </Tooltip>
</td>
{/if}
<div class="absolute left-0 h-7 px-2 py-1" class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {damagePercentage}%"
></div>
