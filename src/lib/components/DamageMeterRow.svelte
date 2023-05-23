<script lang="ts">
    import { EntityType, type Entity } from "$lib/types";
    import { classColors } from "$lib/constants/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";

    export let entity: Entity;
    export let percentage: number;
    export let duration: number;
    export let totalDamageDealt: number;
    export let lastCombatPacket: number;
    export let anyDead: boolean;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportBrand: boolean;

    let color = "#ffffff";
    let alpha = 0.6;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let name: string;
    let damagePercentage: string;
    let damagePercentageRaw: number;
    let deadFor: string;

    $: {
        tweenedValue.set(percentage);
        if (Object.hasOwn(classColors, entity.class)) {
            color = classColors[entity.class].color;
        }
        damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
        damagePercentageRaw = (entity.damageStats.damageDealt / totalDamageDealt) * 100;
        damagePercentage = damagePercentageRaw.toFixed(1);

        if (duration > 0) {
            dps = abbreviateNumberSplit(entity.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general.showNames);
        }
        if (entity.isDead) {
            deadFor = Math.abs((lastCombatPacket - entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
        }
        if (!$settings.meter.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
        }
    }
</script>

<td class="pl-1">
    {#if $settings.general.showEsther && entity.entityType === EntityType.ESTHER}
        <img class="table-cell h-5 w-5" src={$classIconCache[name]} alt={name} use:tooltip={{ content: name }} />
    {:else}
        <img
            class="table-cell h-5 w-5"
            src={$classIconCache[entity.classId]}
            alt={entity.class}
            use:tooltip={{ content: entity.class }} />
    {/if}
</td>
<td colspan="2">
    <div class="truncate">
        {name}
    </div>
</td>
{#if anyDead && $settings.meter.deathTime}
    <td class="px-1 text-center">
        {#if entity.isDead}
            {deadFor}
        {/if}
    </td>
{/if}
{#if $settings.meter.damage}
    <td class="px-1 text-center">
        {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
    </td>
{/if}
{#if $settings.meter.dps}
    <td class="px-1 text-center">
        {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
    </td>
{/if}
{#if damagePercentageRaw < 100 && $settings.meter.damagePercent}
    <td class="px-1 text-center">
        {damagePercentage}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if $settings.meter.critRate}
    <td class="px-1 text-center">
        {round((entity.skillStats.crits / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anyFrontAtk && $settings.meter.frontAtk}
    <td class="px-1 text-center">
        {round((entity.skillStats.frontAttacks / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anyBackAtk && $settings.meter.backAtk}
    <td class="px-1 text-center">
        {round((entity.skillStats.backAttacks / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportBuff && $settings.meter.percentBuffBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBrand && $settings.meter.percentBrand}
    <td class="px-1 text-center">
        {round((entity.damageStats.debuffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if $settings.meter.counters}
    <td class="px-1 text-center">
        {entity.skillStats.counters}<span class="text-3xs text-gray-300" />
    </td>
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    style="background-color: {HexToRgba(color, alpha)}; width: {$tweenedValue}%" />
