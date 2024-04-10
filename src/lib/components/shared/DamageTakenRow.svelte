<script lang="ts">
    import type { Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { localPlayer } from "$lib/utils/stores";

    export let player: Entity;
    export let width: number;
    export let alpha = 0.6;
    export let shadow = false;
    export let tween: boolean;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let color = "#ffffff";
    let name: string;

    let damageTaken: (string | number)[];

    $: {
        tweenedValue.set(width);
        damageTaken = abbreviateNumberSplit(player.damageStats.damageTaken);

        name = formatPlayerName(player, $settings.general.showNames, $settings.general.showGearScore);

        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }
    }
</script>

<td class="pl-1">
    <img
        class="table-cell size-5"
        src={$classIconCache[player.classId]}
        alt={player.class}
        use:tooltip={{ content: player.class }} />
</td>
<td colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: name }}>
            {name}
        </span>
    </div>
</td>
<td class="pl-1 pr-2 text-right" use:tooltip={{ content: player.damageStats.damageTaken.toLocaleString() }}>
    {damageTaken[0]}<span class="text-3xs text-gray-300">{damageTaken[1]}</span>
</td>
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {HexToRgba(color, alpha)}; width: {tween ? $tweenedValue : width}%" />
