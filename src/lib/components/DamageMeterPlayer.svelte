<script lang="ts">
    import type { Entity } from "$lib/types";
    import { classColors } from "$lib/constants/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumber } from "$lib/utils/numbers";


    export let entity: Entity;
    export let percentage: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $: tweenedValue.set(percentage);
    
</script>

<div class="">
    <div class="absolute border border-zinc-800 h-8 px-2 py-1"
        style="background-color: {HexToRgba(classColors[entity.class].color, 0.6)}; width: {$tweenedValue}%"
    ></div>
    <div class="relative flex justify-between h-8 px-2 py-1 text-sm items-center">
        <div class="">
            {entity.class}
        </div>
        <div class="">
            <div class="">
                {abbreviateNumber(entity.damageStats.damageDealt)}
            </div>
        </div>
    </div>
</div>