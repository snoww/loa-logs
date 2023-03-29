<script lang="ts">
    import type { Entity } from "$lib/types";
    import { classColors } from "$lib/constants/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumber, abbreviateNumberSplit } from "$lib/utils/numbers";


    export let entity: Entity;
    export let percentage: number;
    export let duration: number;
    export let totalDamageDealt: number;

    let color = "#000"

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let damageDealt: (string | number)[];
    let dps: (string | number)[];

    $: {
        tweenedValue.set(percentage);
        if (Object.hasOwn(classColors, entity.class)){
            color = classColors[entity.class].color;
        }
        damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
        dps = abbreviateNumberSplit(entity.damageStats.damageDealt / (duration / 1000));
    }
    
</script>

<td class="px-2">
    {entity.class}
</td>
<td class="px-1 text-center">
    {damageDealt[0]}<span class="text-xs text-gray-300">{damageDealt[1]}</span>
</td>
<td class="px-1 text-center">
    {dps[0]}<span class="text-xs text-gray-300">{dps[1]}</span>
</td>
<td class="px-1 text-center">
    {(entity.damageStats.damageDealt / totalDamageDealt * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<div class="absolute left-0 border border-zinc-800 h-8 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>