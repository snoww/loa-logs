<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { Tooltip } from 'flowbite-svelte';
    import BuffTooltipDetail from "./shared/BuffTooltipDetail.svelte";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName } from "$lib/utils/strings";


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
        if (Object.hasOwn(classColors, player.class)){
            color = classColors[player.class].color;
        }

        playerName = formatPlayerName(player, $settings.general.showNames);

        if (groupedSynergies.size > 0) {
            synergyPercentageDetails = [];
            groupedSynergies.forEach((synergies, _) => {
                let synergyDamage = 0;
                let buff = new BuffDetails();
                synergies.forEach((syn, id) => {
                    if (player.damageStats.buffedBy[id]) {
                        buff.buffs.push(new Buff(syn.source.icon, (player.damageStats.buffedBy[id] / player.damageStats.damageDealt * 100).toFixed(1), syn.source.skill?.icon));
                        synergyDamage += player.damageStats.buffedBy[id];
                    } else if (player.damageStats.debuffedBy[id]) {
                        buff.buffs.push(new Buff(syn.source.icon, (player.damageStats.debuffedBy[id] / player.damageStats.damageDealt * 100).toFixed(1), syn.source.skill?.icon));
                        synergyDamage += player.damageStats.debuffedBy[id];
                    }
                });

                if (synergyDamage > 0) {
                    buff.percentage = (synergyDamage / player.damageStats.damageDealt * 100).toFixed(1);
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
    <img class="h-5 w-5 table-cell" src={$classIconCache[player.classId]} alt={player.class} />
</td>
<td class="">
    <div class="truncate">
        {playerName}
    </div>
</td>
{#if groupedSynergies.size > 0}
{#each synergyPercentageDetails as synergy}
    <td class="px-1 text-center">
        {#if synergy.percentage}
        <div>
            {synergy.percentage}<span class="text-3xs text-gray-300">%</span>
        </div>
        <Tooltip placement="bottom" defaultClass="bg-zinc-900 p-2 text-gray-300 z-50">
            <BuffTooltipDetail buffDetails={synergy} />
        </Tooltip>
        {/if}
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, alpha)}; width: {$tweenedValue}%"
></div>