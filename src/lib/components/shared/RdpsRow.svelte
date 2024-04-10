<script lang="ts">
    import { tooltip } from "$lib/utils/tooltip";
    import { classIconCache, colors, settings } from "$lib/utils/settings";
    import type { Entity } from "$lib/types";
    import { formatPlayerName } from "$lib/utils/strings";
    import { abbreviateNumberSplit, getBaseDamage, getRDamage } from "$lib/utils/numbers";
    import { HexToRgba } from "$lib/utils/colors";
    import { localPlayer } from "$lib/utils/stores";

    export let player: Entity;
    export let totalDamageDealt: number;
    export let width: number;
    export let shadow: boolean = false;
    export let alpha: number = 0.6;
    export let duration: number;
    export let meterSettings: any;

    let playerName: string;
    let color = "#ffffff";
    let damageDealt: (string | number)[];
    let damageGiven: (string | number)[];
    let damageReceived: (string | number)[];
    let damagePercentage: string;
    let rDamage: number;
    let rDps: (string | number)[];
    let sSynPercentage = "0.0";
    let dSynPercentage = "0.0";
    let synPercentage = "0.0";
    $: {
        rDamage = getRDamage(player.damageStats);
        rDps = abbreviateNumberSplit(rDamage / (duration / 1000));
        damageDealt = abbreviateNumberSplit(rDamage);
        damagePercentage = ((rDamage / totalDamageDealt) * 100).toFixed(1);
        damageGiven = abbreviateNumberSplit(player.damageStats.rdpsDamageGiven);
        damageReceived = abbreviateNumberSplit(player.damageStats.rdpsDamageReceived);
        let baseDamage = getBaseDamage(player.damageStats);
        let sSyn = player.damageStats.rdpsDamageReceivedSupport / baseDamage;
        let dSyn = (player.damageStats.rdpsDamageReceived - player.damageStats.rdpsDamageReceivedSupport) / baseDamage;
        let syn = sSyn + dSyn;
        sSynPercentage = (sSyn * 100).toFixed(1);
        dSynPercentage = (dSyn * 100).toFixed(1);
        synPercentage = (syn * 100).toFixed(1);

        playerName = formatPlayerName(player, $settings.general.showNames, $settings.general.showGearScore);
        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }
    }
</script>

<tr class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
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
    <td class="px-1 text-center">
        {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
    </td>
    <td class="px-1 text-center">
        {rDps[0]}<span class="text-3xs text-gray-300">{rDps[1]}</span>
    </td>
    <td class="px-1 text-center">
        {damagePercentage}<span class="text-xs text-gray-300">%</span>
    </td>
    {#if meterSettings.rdpsDamageGiven}
        <td class="px-1 text-center">
            {damageReceived[0]}<span class="text-3xs text-gray-300">{damageReceived[1]}</span>
        </td>
    {/if}
    {#if meterSettings.rdpsDamageReceived}
        <td class="px-1 text-center">
            {damageGiven[0]}<span class="text-3xs text-gray-300">{damageGiven[1]}</span>
        </td>
    {/if}
    <td class="px-1 text-center">
        {synPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    <td class="px-1 text-center">
        {sSynPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    <td class="px-1 text-center">
        {dSynPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    <div
        class="absolute left-0 -z-10 h-7 px-2 py-1"
        class:shadow-md={shadow}
        style="background-color: {HexToRgba(color, alpha)}; width: {width}%" />
</tr>
