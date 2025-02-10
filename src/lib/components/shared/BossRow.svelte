<script lang="ts">
    import type { Entity } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    interface Props {
        boss: Entity;
        width: number;
        shadow?: boolean;
        tween: boolean;
        duration: number;
        index: number;
    }

    let { boss, width, shadow = false, tween, duration, index }: Props = $props();

    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(width ?? 0);
    });

    let color = "#164e63";

    let damageDealt: (string | number)[] = $state(abbreviateNumberSplit(boss.damageStats.damageDealt));
    let dps: (string | number)[] = $state([]);

    $effect(() => {
        if (duration > 0) {
            dps = abbreviateNumberSplit(boss.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }
    });
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
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {tween ? tweenedValue.current : width}%"></td>
