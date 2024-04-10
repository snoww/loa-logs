<script lang="ts">
    import type { Entity } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";

    export let boss: Entity;
    export let width: number;
    export let shadow = false;
    export let tween: boolean;
    export let duration: number;
    export let index: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let color = "#164e63";

    let damageDealt: (string | number)[];
    let dps: (string | number)[];

    $: {
        tweenedValue.set(width);
        damageDealt = abbreviateNumberSplit(boss.damageStats.damageDealt);

        if (duration > 0) {
            dps = abbreviateNumberSplit(boss.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }
    }
</script>

<td colspan="2" class="px-2">
    <div class="truncate">
        <span use:tooltip={{ content: boss.name }}>
            {boss.name}
        </span>
    </div>
</td>
<td class="px-1 text-center" use:tooltip={{ content: boss.damageStats.damageDealt.toLocaleString() }}>
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td>
<td class="px-1 text-center">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {tween ? $tweenedValue : width}%"  />
