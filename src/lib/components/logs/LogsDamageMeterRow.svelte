<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { isValidName } from "$lib/utils/strings";

    export let entity: Entity;
    export let percentage: number;
    export let icon: string;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let end: number;

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let damagePercentage: number;
    let playerName: string;
    let deadFor: string;

    let color = "#ffffff"

    if (Object.hasOwn(classColors, entity.class)){
        color = classColors[entity.class].color;
    }
    damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    damagePercentage = entity.damageStats.damageDealt / totalDamageDealt * 100;
    
    dps = abbreviateNumberSplit(entity.damageStats.dps);

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
        
        deadFor = ((end - entity.lastUpdate) / 1000).toFixed(0) + "s";
    }
</script>

<td class="px-1 relative z-10">
    <div class="flex space-x-1">
        <img class="h-5 w-5" src={icon} alt={entity.class} />
        <div class="truncate">
            {playerName}
        </div>
    </div>
</td>
<td class="px-1 text-center relative z-10" class:hidden={!anyDead}>
    {entity.isDead ? deadFor : ""}
</td>
<td class="px-1 text-center relative z-10">
    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {damagePercentage.toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.crits / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.frontAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {(entity.skillStats.backAttacks / entity.skillStats.hits * 100).toFixed(1)}<span class="text-3xs text-gray-300">%</span>
</td>
<div class="absolute left-0 h-7 px-2 py-1 z-0"
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"
></div>