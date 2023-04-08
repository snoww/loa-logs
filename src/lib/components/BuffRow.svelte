<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { Tooltip } from 'flowbite-svelte';
    import BuffTooltipDetail from "./shared/BuffTooltipDetail.svelte";


    export let player: Entity;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let percentage: number;

    let color = "#ffffff"
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

        playerName = player.name;
        if (player.class) {
            playerName += ` (${player.class})`;
        }
        if (player.isDead) {
            playerName = "💀 " + playerName;
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
{#each synergyPercentageDetails as synergy}
    <td class="px-1 text-center">
        <div>
            {synergy.percentage}<span class="text-3xs text-gray-300" class:hidden={!synergy.percentage}>%</span>
        </div>
        <Tooltip placement="bottom" defaultClass="bg-zinc-900 p-2 text-gray-300 z-50">
            <BuffTooltipDetail buffDetails={synergy} />
        </Tooltip>
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>