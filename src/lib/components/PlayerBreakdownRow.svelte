<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { Tooltip } from "flowbite-svelte";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";


    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;
    export let duration: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    })

    let critPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    $: {
        tweenedValue.set(damagePercentage);
        if (skill.hits !== 0) {
            critPercentage = (skill.crits / skill.hits * 100).toFixed(1);
            faPercentage = (skill.frontAttacks / skill.hits * 100).toFixed(1) ;
            baPercentage = (skill.backAttacks / skill.hits * 100).toFixed(1);
        }        
    }
    
    async function getSkillIconPath() {
        if (skill.icon.startsWith("http")) {
            return skill.icon;
        }
        let fileName;
        if (skill.icon) {
            fileName = skill.icon;
        } else {
            fileName = "unknown.png";
        }
        return convertFileSrc(await join(await resourceDir(), 'images', 'skills', fileName));
    }

</script>


<td class="pl-1">
    {#await getSkillIconPath()}
        <img class="h-5 w-5" src="" alt={skill.name} />
    {:then path} 
        <img class="h-5 w-5" src={path} alt={skill.name} />
    {/await}
</td>
<td>
    <div class="truncate">
        {skill.name}
    </div>
</td>
{#if $settings.meter.breakdown.damage}
<td class="px-1 text-center">
    {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
</td>
{/if}
{#if $settings.meter.breakdown.dps}
<td class="px-1 text-center">
    {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
</td>
{/if}
{#if $settings.meter.breakdown.damagePercent}
<td class="px-1 text-center">
    {(skill.totalDamage / playerDamageDealt * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.breakdown.critRate}
<td class="px-1 text-center">
    {critPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if hasFrontAttacks && $settings.meter.breakdown.frontAtk}
<td class="px-1 text-center">
    {faPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if hasBackAttacks && $settings.meter.breakdown.backAtk}
<td class="px-1 text-center">
    {baPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.breakdown.avgDamage}
<td class="px-1 text-center relative z-10">
    {abbreviateNumberSplit(skill.totalDamage / skill.hits)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.totalDamage / skill.hits)[1]}</span>
</td>
{/if}
{#if $settings.meter.breakdown.maxDamage}
<td class="px-1 text-center relative z-10">
    {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300">{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
</td>
{/if}
{#if $settings.meter.breakdown.casts}
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
{#if $settings.meter.breakdown.hits}
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
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>
