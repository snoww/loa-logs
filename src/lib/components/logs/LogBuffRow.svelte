<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { formatPlayerName } from "$lib/utils/strings";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import BuffTooltipDetail from "../shared/BuffTooltipDetail.svelte";


    export let player: Entity;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let percentage: number;

    let color = "#ffffff"
    let playerName: string;
    let synergyPercentageDetails: Array<BuffDetails>;

    if (Object.hasOwn(classColors, player.class)){
        color = classColors[player.class].color;
    }

    $: {
        playerName = formatPlayerName(player, $settings.general.showNames);
    }

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
    
</script>

<td class="relative z-10 pl-1">
    <img class="h-5 w-5 table-cell" src={$classIconCache[player.classId]} alt={player.class} />
</td>
<td class="relative z-10">
    <div class="truncate">
        {playerName}
    </div>
</td>
{#if groupedSynergies.size > 0}
{#each synergyPercentageDetails as synergy}
    <td class="px-1 text-center text-3xs">
        {#if synergy.percentage}
        <BuffTooltipDetail {synergy} />
        {/if}
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 z-0" class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"
></div>
