<script lang="ts">
    import ShieldTooltipDetail from "$lib/components/shared/ShieldTooltipDetail.svelte";
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import { type Entity, ShieldDetails } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { classIconCache } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";

    interface Props {
        enc: EncounterState;
        player: Entity;
        playerShields: Array<ShieldDetails>;
        percentage: number;
    }

    let { enc, player, playerShields, percentage }: Props = $props();
    let entityState = $derived(new EntityState(player, enc));

    let totalShield = $derived(playerShields.reduce((acc, buff) => acc + buff.total, 0));
    let totalShieldStr = $derived(abbreviateNumberSplit(totalShield));
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
        <span use:tooltip={{ content: entityState.name }}>
            {entityState.name}
        </span>
    </div>
</td>
<td class="text-3xs px-1 text-center" use:tooltip={{ content: totalShield.toLocaleString() }}>
    {totalShieldStr[0]}<span class="text-3xs text-gray-300">{totalShieldStr[1]}</span>
</td>
{#if playerShields.length > 0}
    {#each playerShields as shield (shield.id)}
        <td class="text-3xs px-1 text-center">
            {#if shield.total}
                <ShieldTooltipDetail shieldDetails={shield} />
            {/if}
        </td>
    {/each}
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(entityState.color, 0.6)}; width: {percentage}%"></td>
