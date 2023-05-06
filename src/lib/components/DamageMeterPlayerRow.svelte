<script lang="ts">
    import { EntityType, type Entity } from "$lib/types";
    import { classColors } from "$lib/constants/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";
    import { classIconCache, settings } from "$lib/utils/settings";

    export let entity: Entity;
    export let percentage: number;
    export let duration: number;
    export let totalDamageDealt: number;
    export let lastCombatPacket: number;
    export let anyDead: boolean;

    let color = "#ffffff"

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let name: string;
    let damagePercentage: string;
    let deadFor: string;

    $: {
        tweenedValue.set(percentage);
        if (Object.hasOwn(classColors, entity.class)){
            color = classColors[entity.class].color;
        }
        damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
        damagePercentage = (entity.damageStats.damageDealt / totalDamageDealt * 100).toFixed(1);        
        
        if (duration > 0) {
            dps = abbreviateNumberSplit(entity.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general.showNames);
        }
        if (entity.isDead) {
            deadFor = (((lastCombatPacket - entity.damageStats.deathTime) / 1000).toFixed(0) + "s").replace('-', '');
        }             
    }
        
</script>

<td class="px-1">
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
{#if anyDead && $settings.meter.deathTime}
<td class="px-1 text-center relative z-10">
    {entity.isDead ? deadFor : ""}
</td>
{/if}
{#if $settings.meter.damage}
<td class="px-1 text-center">
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td>
{/if}
{#if $settings.meter.dps}
<td class="px-1 text-center">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
{/if}
{#if damagePercentage !== "100.0" && $settings.meter.damagePercent}
<td class="px-1 text-center">
    {damagePercentage}<span class="text-xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.critRate}
<td class="px-1 text-center">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.frontAtk}
<td class="px-1 text-center">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.backAtk}
<td class="px-1 text-center">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if $settings.meter.counters}
<td class="px-1 text-center">
    {entity.skillStats.counters}<span class="text-3xs text-gray-300"></span>
</td>
{/if}
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>