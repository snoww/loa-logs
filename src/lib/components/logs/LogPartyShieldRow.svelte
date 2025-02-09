<script lang="ts">
    import { HexToRgba } from "$lib/utils/colors";
    import { formatPlayerName } from "$lib/utils/strings";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { localPlayer, takingScreenshot } from "$lib/utils/stores";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { type Entity, ShieldDetails } from "$lib/types";
    import ShieldTooltipDetail from "$lib/components/shared/ShieldTooltipDetail.svelte";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";

    interface Props {
        player: Entity;
        playerShields: Array<ShieldDetails>;
        percentage: number;
    }

    let { player, playerShields, percentage }: Props = $props();

    let color = $state("#ffffff");
    let playerName: string = $derived(formatPlayerName(player, $settings.general));

    if (Object.hasOwn($colors, player.class)) {
        if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
            color = $colors["Local"].color;
        } else {
            color = $colors[player.class].color;
        }
    }

    let totalShieldStr: (string | number)[] = $derived.by(() => {
        let totalShield = playerShields.reduce((acc, buff) => acc + buff.total, 0);
        return abbreviateNumberSplit(totalShield);
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
        <span use:tooltip={{ content: playerName }}>
            {playerName}
        </span>
    </div>
</td>
<td class="px-1 text-center text-3xs">
    {totalShieldStr[0]}<span class="text-3xs text-gray-300">{totalShieldStr[1]}</span>
</td>
{#if playerShields.length > 0}
    {#each playerShields as shield (shield.id)}
        <td class="px-1 text-center text-3xs">
            {#if shield.total}
                <ShieldTooltipDetail shieldDetails={shield} />
            {/if}
        </td>
    {/each}
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"></td>
