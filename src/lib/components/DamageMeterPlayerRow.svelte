<script lang="ts">
    import type { Entity } from "$lib/types";
    import { classColors } from "$lib/constants/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { isValidName } from "$lib/utils/strings";

    export let entity: Entity;
    export let percentage: number;
    export let duration: number;
    export let totalDamageDealt: number;
    export let lastCombatPacket: number;

    let color = "#ffffff"

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let playerName: string;
    let damagePercentage: number;
    let deadFor: string;

    $: {
        tweenedValue.set(percentage);
        if (Object.hasOwn(classColors, entity.class)){
            color = classColors[entity.class].color;
        }
        damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
        damagePercentage = entity.damageStats.damageDealt / totalDamageDealt * 100;
        
        if (duration > 0) {
            dps = abbreviateNumberSplit(entity.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }

        playerName = entity.name;
        // todo use settings
        if (!isValidName(playerName)) {
            playerName = "";
            // playerName += " ("
            if (entity.gearScore > 0) {
                playerName += entity.gearScore + " ";
            }
            if (entity.class) {
                playerName += entity.class;
            }
            // playerName += ")";
        }
        if (entity.isDead) {
            playerName = "💀 " + playerName;
            deadFor = ((lastCombatPacket - entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
        }        
    }

    async function getClassIconPath() {
        let path;
        if (entity.classId > 100) {
            path = `${entity.classId}.png`;
        } else {
            path = `${1}/101.png`;
        }
        return convertFileSrc(await join(await resourceDir(), 'images', 'classes', path));
    }
        
</script>

<td class="px-1">
    <div class="flex space-x-1">
        {#await getClassIconPath()}
            <img class="h-5 w-5" src="" alt={entity.class} />
        {:then path} 
            <img class="h-5 w-5" src={path} alt={entity.class} />
        {/await}
        <div class="truncate">
            {playerName}
        </div>
    </div>
</td>
<!-- <td class="px-1 text-center">
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td> -->
<td class="px-1 text-center">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
{#if damagePercentage < 100}
<td class="px-1 text-center">
    {damagePercentage.toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
{/if}
<td class="px-1 text-center">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<div class="absolute left-0 h-7 px-2 py-1 -z-10"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>