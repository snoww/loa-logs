<script lang="ts">
    import { run } from "svelte/legacy";

    import { tooltip } from "$lib/utils/tooltip";
    import { classIconCache, colors, settings } from "$lib/utils/settings";
    import type { Entity } from "$lib/types";
    import { formatPlayerName } from "$lib/utils/strings";
    import { abbreviateNumberSplit, getBaseDamage, getRDamage } from "$lib/utils/numbers";
    import { HexToRgba } from "$lib/utils/colors";
    import { localPlayer } from "$lib/utils/stores";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    interface Props {
        player: Entity;
        totalDamageDealt: number;
        width: number;
        shadow?: boolean;
        alpha?: number;
        duration: number;
        meterSettings: any;
        isLiveMeter?: boolean;
    }

    let {
        player,
        totalDamageDealt,
        width,
        shadow = false,
        alpha = 0.6,
        duration,
        meterSettings,
        isLiveMeter = false
    }: Props = $props();

    let playerName: string = $derived(formatPlayerName(player, $settings.general));
    let tooltipName: string = $state("");
    let color = $state("#ffffff");
    let damageDealt: (string | number)[] = $state([]);
    let damageGiven: (string | number)[] = $state([]);
    let damageReceived: (string | number)[] = $state([]);
    let damagePercentage: string = $state("");
    let rDamage: number = $state(0);
    let rDps: (string | number)[] = $state([]);
    let sSynPercentage = $state("0.0");
    let dSynPercentage = $state("0.0");
    let synPercentage = $state("0.0");
    let sConPercentage = $state("0.0");
    let dConPercentage = $state("0.0");
    let conPercentage = $state("0.0");
    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(width ?? 0);
    });

    $effect.pre(() => {
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
        sConPercentage = ((1 - 1 / (1 + sSyn)) * 100).toFixed(1);
        dConPercentage = ((1 - 1 / (1 + dSyn)) * 100).toFixed(1);
        conPercentage = ((1 - 1 / (1 + syn)) * 100).toFixed(1);
    });

    $effect(() => {
        if ($settings.general.showNames) {
            tooltipName = player.name;
        } else {
            tooltipName = player.class;
        }
    });

    $effect(() => {
        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }
    });
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
    {#if meterSettings.rdpsContribution}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `${conPercentage}% of <span class="italic">${tooltipName}'s</span> damage is from all synergies`
            }}>
            {conPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    {#if meterSettings.rdpsSContribution}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `${sConPercentage}% of <span class="italic">${tooltipName}'s</span> damage is from supports`
            }}>
            {sConPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    {#if meterSettings.rdpsDContribution}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `${dConPercentage}% of <span class="italic">${tooltipName}'s</span> damage is from dealers`
            }}>
            {dConPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    {#if meterSettings.rdpsSyn}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `<span class="italic">${tooltipName}</span> dealt +${synPercentage}% more damage from all buffs`
            }}>
            {synPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    {#if meterSettings.rdpsSSyn}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `<span class="italic">${tooltipName}</span> dealt +${sSynPercentage}% more damage from support buffs`
            }}>
            {sSynPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    {#if meterSettings.rdpsDSyn}
        <td
            class="px-1 text-center"
            use:tooltip={{
                content: `<span class="italic">${tooltipName}</span> dealt +${dSynPercentage}% more damage from dealer buffs`
            }}>
            {dSynPercentage}<span class="text-3xs text-gray-300">%</span>
        </td>
    {/if}
    <td
        class="absolute left-0 -z-10 h-7 px-2 py-1"
        class:shadow-md={shadow}
        style="background-color: {HexToRgba(color, alpha)}; width: {isLiveMeter ? tweenedValue.current : width}%"></td>
</tr>
