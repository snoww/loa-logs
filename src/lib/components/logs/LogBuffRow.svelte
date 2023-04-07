<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";

    export let player: Entity;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let percentage: number;
    export let classIconsCache: { [key: number]: string };

    let color = "#ffffff"
    let playerName: string;
    let synergyPercentages: Array<string>;


    if (Object.hasOwn(classColors, player.class)){
        color = classColors[player.class].color;
    }

    playerName = player.name;
    if (player.class) {
        playerName += ` (${player.class})`;
    }
    if (player.isDead) {
        playerName = "💀 " + playerName;
    }

    if (groupedSynergies.size > 0) {
        synergyPercentages = [];
        groupedSynergies.forEach((synergies, _) => {
            let synergyDamage = 0;
            synergies.forEach((_, id) => {
                if (player.damageStats.buffedBy[id]) {
                    synergyDamage += player.damageStats.buffedBy[id];
                } else if (player.damageStats.debuffedBy[id]) {
                    synergyDamage += player.damageStats.debuffedBy[id];
                }
            });

            if (synergyDamage == 0) {
                synergyPercentages.push("");
            } else {
                synergyPercentages.push((synergyDamage / player.damageStats.damageDealt * 100).toFixed(1));
            }
        });
    }   

</script>

<td class="pl-1 relative z-10">
    <img class="h-5 w-5" src={classIconsCache[player.classId]} alt={player.class} />
</td>
<td class="relative z-10">
    <div class="truncate">
        {playerName}
    </div>
</td>
{#if groupedSynergies.size > 0}
{#each synergyPercentages as percentage}
    <td class="px-1 text-center relative z-10">
        {percentage}<span class="text-3xs text-gray-300" class:hidden={!percentage}>%</span>
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 z-0"
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"
></div>