<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";


    export let player: Entity;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let percentage: number;

    let color = "#ffffff"
    let playerName: string;
    let synergyPercentages: Array<string>;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $: {
        tweenedValue.set(percentage);
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
    }

    async function getClassIconPath() {
        let path;
        if (player.classId > 100) {
            path = `${player.classId}.png`;
        } else {
            path = `${1}/101.png`;
        }
        return convertFileSrc(await join(await resourceDir(), 'images', 'classes', path));
    }

</script>

<td class="pl-1">
    {#await getClassIconPath()}
        <img class="h-5 w-5" src="" alt={player.class} />
    {:then path} 
        <img class="h-5 w-5" src={path} alt={player.class} />
    {/await}
</td>
<td class="">
    <div class="truncate">
        {playerName}
    </div>
</td>
{#if groupedSynergies.size > 0}
{#each synergyPercentages as percentage}
    <td class="px-1 text-center">
        {percentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>