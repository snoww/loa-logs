<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { formatPlayerName } from "$lib/utils/strings";
    import type { Writable } from "svelte/store";

    export let entity: Entity;
    export let percentage: number;
    export let icon: string;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let end: number;
    export let hideNames: Writable<boolean>;

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let damagePercentage: number;
    let playerName: string;
    let deadFor: string;    

    let color = "#ffffff"

    if (Object.hasOwn(classColors, entity.class)){
        color = classColors[entity.class].color;
    }
    damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    damagePercentage = entity.damageStats.damageDealt / totalDamageDealt * 100;
    
    dps = abbreviateNumberSplit(entity.damageStats.dps);

    $: {
        playerName = formatPlayerName(entity, $hideNames)
    }
    if (entity.isDead) {       
        deadFor = ((end - entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
    }
</script>

<td class="px-1 relative z-10">
    <div class="flex space-x-1">
        <img class="h-5 w-5" src={icon} alt={entity.class} />
        <div class="truncate pl-px">
            {playerName}
        </div>
    </div>
</td>
{#if anyDead}
<td class="px-1 text-center relative z-10">
    {entity.isDead ? deadFor : ""}
</td>
{/if}
<td class="px-1 text-center relative z-10">
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {damagePercentage.toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<div class="absolute left-0 h-7 px-2 py-1 z-0 shadow-md"
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"
></div>