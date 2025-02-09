<script lang="ts">
    import { HexToRgba } from "$lib/utils/colors";
    import { formatPlayerName } from "$lib/utils/strings";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { localPlayer, takingScreenshot } from "$lib/utils/stores";
    import BuffTooltipDetail from "../shared/BuffTooltipDetail.svelte";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import type { BuffDetails, Entity } from "$lib/types";

    interface Props {
        player: Entity;
        playerBuffs: Array<BuffDetails>;
        percentage: number;
    }

    let { player, playerBuffs, percentage }: Props = $props();

    let color = $state("#ffffff");
    let playerName: string = $derived(formatPlayerName(player, $settings.general));

    if (Object.hasOwn($colors, player.class)) {
        if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
            color = $colors["Local"].color;
        } else {
            color = $colors[player.class].color;
        }
    }
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
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"></td>
