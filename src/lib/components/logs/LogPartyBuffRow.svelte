<script lang="ts">
    import { HexToRgba } from "$lib/utils/colors";
    import { formatPlayerName } from "$lib/utils/strings";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import BuffTooltipDetail from "../shared/BuffTooltipDetail.svelte";
    import { tooltip } from "$lib/utils/tooltip";
    import type { BuffDetails, Entity } from "$lib/types";

    export let player: Entity;
    export let playerBuffs: Array<BuffDetails>;
    export let percentage: number;

    let color = "#ffffff";
    let playerName: string;

    if (Object.hasOwn($colors, player.class)) {
        color = $colors[player.class].color;
    }

    $: {
        playerName = formatPlayerName(player, $settings.general.showNames, $settings.general.showGearScore);
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
        <span use:tooltip={{ content: playerName }}>
            {playerName}
        </span>
    </div>
</td>
{#if playerBuffs.length > 0}
    {#each playerBuffs as buff (buff.id)}
        <td class="px-1 text-center text-3xs">
            {#if buff.percentage}
                <BuffTooltipDetail synergy={buff} />
            {/if}
        </td>
    {/each}
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%" />
