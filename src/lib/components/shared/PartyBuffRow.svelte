<script lang="ts">
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import type { BuffDetails, Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { classIconCache } from "$lib/utils/settings";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import BuffTooltipDetail from "./BuffTooltipDetail.svelte";

    interface Props {
        player: Entity;
        enc: EncounterState;
        playerBuffs: Array<BuffDetails>;
        percentage: number;
    }

    let { player, enc, playerBuffs, percentage }: Props = $props();
    let entityState = $derived(new EntityState(player, enc));

    const tweenedValue = new Tween(enc.live ? 0 : percentage, {
        duration: 400,
        easing: cubicOut
    });

    let alpha = $derived(enc.live && !enc.curSettings.showClassColors ? 0 : 0.6);

    $effect(() => {
        tweenedValue.set(percentage ?? 0);
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
        <span use:tooltip={{ content: entityState.name }}>
            {entityState.name}
        </span>
    </div>
</td>
{#if playerBuffs.length > 0}
    {#each playerBuffs as buff (buff.id)}
        <td class="text-3xs px-1 text-center">
            {#if buff.percentage}
                <BuffTooltipDetail synergy={buff} />
            {/if}
        </td>
    {/each}
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    style="background-color: {HexToRgba(entityState.color, alpha)}; width: {tweenedValue.current}%"></td>
