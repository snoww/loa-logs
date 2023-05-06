<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { EntityType, type Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";

    export let entity: Entity;
    export let percentage: number;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let end: number;

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let damagePercentage: string;
    let name: string;
    let deadFor: string;    

    let color = "#ffffff"

    if (Object.hasOwn(classColors, entity.class)){
        color = classColors[entity.class].color;
    }
    damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    damagePercentage = (entity.damageStats.damageDealt / totalDamageDealt * 100).toFixed(1);
    
    dps = abbreviateNumberSplit(entity.damageStats.dps);

    $: {
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general.showNames);
        }
    }
    if (entity.isDead) {       
        deadFor = (((end - entity.damageStats.deathTime) / 1000).toFixed(0) + "s").replace('-', '');
    }
</script>

<td class="px-1 relative z-10">
    <div class="flex space-x-1">
        {#if $settings.general.showEsther && entity.entityType === EntityType.ESTHER}
        <img class="h-5 w-5" src={$classIconCache[name]} alt={name} />
        <div class="truncate pl-px">
            {name}
        </div>
        {:else}
        <img class="h-5 w-5" src={$classIconCache[entity.classId]} alt={entity.class} />
        <div class="truncate pl-px">
            {name}
        </div>
        {/if}
    </div>
</td>
{#if anyDead && $settings.logs.deathTime}
<td class="px-1 text-center relative z-10">
    {entity.isDead ? deadFor : ""}
</td>
{/if}
{#if $settings.logs.damage}
<td class="px-1 text-center relative z-10">
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td>
{/if}
{#if $settings.logs.dps}
<td class="px-1 text-center relative z-10">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
{/if}
{#if damagePercentage !== "100.0" && $settings.logs.damagePercent}
<td class="px-1 text-center relative z-10">
    {damagePercentage}<span class="text-xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.critRate}
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.frontAtk}
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.backAtk}
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.logs.counters}
<td class="px-1 text-center relative z-10">
    {entity.skillStats.counters}<span class="text-3xs text-gray-300"></span>
</td>
{/if}
<div class="absolute left-0 h-7 px-2 py-1 z-0" class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"
></div>