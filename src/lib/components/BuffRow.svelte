<script lang="ts">
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import BuffTooltipDetail from "./shared/BuffTooltipDetail.svelte";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";
    import { round } from "$lib/utils/numbers";
    import { addBardBubbles } from "$lib/utils/buffs";
    import { localPlayer } from "$lib/utils/stores";

    export let player: Entity;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let percentage: number;

    let color = "#ffffff";
    let alpha = 0.6;
    let playerName: string;
    let synergyPercentageDetails: Array<BuffDetails>;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $: {
        tweenedValue.set(percentage);
        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }

        playerName = formatPlayerName(player, $settings.general.showNames, $settings.general.showGearScore);

        if (groupedSynergies.size > 0) {
            synergyPercentageDetails = [];
            groupedSynergies.forEach((synergies, key) => {
                let synergyDamage = 0;
                let buff = new BuffDetails();
                synergies.forEach((syn, id) => {
                    if (player.damageStats.buffedBy[id]) {
                        let b = new Buff(
                            syn.source.icon,
                            round((player.damageStats.buffedBy[id] / player.damageStats.damageDealt) * 100),
                            syn.source.skill?.icon
                        );
                        addBardBubbles(key, b, syn);
                        buff.buffs.push(b);
                        synergyDamage += player.damageStats.buffedBy[id];
                    } else if (player.damageStats.debuffedBy[id]) {
                        buff.buffs.push(
                            new Buff(
                                syn.source.icon,
                                round((player.damageStats.debuffedBy[id] / player.damageStats.damageDealt) * 100),
                                syn.source.skill?.icon
                            )
                        );
                        synergyDamage += player.damageStats.debuffedBy[id];
                    }
                });

                if (synergyDamage > 0) {
                    buff.percentage = round((synergyDamage / player.damageStats.damageDealt) * 100);
                }
                synergyPercentageDetails.push(buff);
            });
        }

        if (!$settings.meter.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
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
        <span use:tooltip={{ content: playerName }}>
            {playerName}
        </span>
    </div>
</td>
{#if groupedSynergies.size > 0}
    {#each synergyPercentageDetails as synergy}
        <td class="px-1 text-center">
            {#if synergy.percentage}
                <BuffTooltipDetail {synergy} />
            {/if}
        </td>
    {/each}
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    style="background-color: {HexToRgba(color, alpha)}; width: {$tweenedValue}%" />
