<script lang="ts">
    import type { Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName } from "$lib/utils/strings";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { localPlayer } from "$lib/utils/stores";
    import { Tween } from "svelte/motion";

    interface Props {
        player: Entity;
        width: number;
        alpha?: number;
        shadow?: boolean;
        tween: boolean;
    }

    let { player, width, alpha = 0.6, shadow = false, tween }: Props = $props();

    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(width  ?? 0);
    });

    let color = $state("#ffffff");
    let name: string = $state(formatPlayerName(player, $settings.general));

    let damageTaken: (string | number)[] = $state(abbreviateNumberSplit(player.damageStats.damageTaken));

    $effect(() => {
        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }
    });
</script>

<td class="pl-1">
    <img
        class="table-cell size-5"
        src={$classIconCache[player.classId]}
        alt={player.class}
        use:tooltip={{ content: generateClassTooltip(player) }} />
</td>
<td colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: name }}>
            {name}
        </span>
    </div>
</td>
<td class="pl-1 pr-2 text-right">
    <span use:tooltip={{ content: player.damageStats.damageTaken.toLocaleString() }}>
        {damageTaken[0]}<span class="text-3xs text-gray-300">{damageTaken[1]}</span>
    </span>
</td>
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {HexToRgba(color, alpha)}; width: {tween ? tweenedValue.current : width}%"></td>
