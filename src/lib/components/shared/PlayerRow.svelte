<script lang="ts">
    import { EntityType, type Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";

    export let entity: Entity;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportBrand: boolean;
    export let end: number;
    export let dps: (string | number)[];

    export let alpha: number = 0.6;
    export let width: number;
    export let meterSettings: any;
    export let shadow: boolean = false;
    export let isSolo: boolean;

    let damageDealt: (string | number)[];
    let damagePercentage: string;
    let name: string;
    let color = "#ffffff";
    let deadFor: string;

    let critPercentage = "0.0";
    let critDmgPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    $: {
        damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
        damagePercentage = ((entity.damageStats.damageDealt / totalDamageDealt) * 100).toFixed(1);

        if (entity.skillStats.hits !== 0) {
            critDmgPercentage = round((entity.damageStats.critDamage / entity.damageStats.damageDealt) * 100);
            critPercentage = round((entity.skillStats.crits / entity.skillStats.hits) * 100);
            if (meterSettings.positionalDmgPercent && (entity.damageStats.frontAttackDamage > 0 || entity.damageStats.backAttackDamage > 0)) {
                faPercentage = round((entity.damageStats.frontAttackDamage / entity.damageStats.damageDealt) * 100);
                baPercentage = round((entity.damageStats.backAttackDamage / entity.damageStats.damageDealt) * 100);
            } else {
                faPercentage = round((entity.skillStats.frontAttacks / entity.skillStats.hits) * 100);
                baPercentage = round((entity.skillStats.backAttacks / entity.skillStats.hits) * 100);
            }
        }

        if (Object.hasOwn($colors, entity.class)) {
            color = $colors[entity.class].color;
        }
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general.showNames, $settings.general.showGearScore);
        }
        if (entity.isDead) {
            deadFor = Math.abs((end - entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
        }
    }
</script>

<td class="pl-1">
    {#if $settings.general.showEsther && entity.entityType === EntityType.ESTHER}
        <img class="table-cell size-5" src={$classIconCache[name]} alt={name} use:tooltip={{ content: name }} />
    {:else}
        <img
            class="table-cell size-5"
            src={$classIconCache[entity.classId]}
            alt={entity.class}
            use:tooltip={{ content: entity.class }} />
    {/if}
</td>
<td colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: name }}>
            {name}
        </span>
    </div>
</td>
{#if anyDead && meterSettings.deathTime}
    <td class="px-1 text-center">
        {#if entity.isDead}
            {deadFor}
        {/if}
    </td>
{/if}
{#if meterSettings.damage}
    <td class="px-1 text-center" use:tooltip={{ content: entity.damageStats.damageDealt.toLocaleString() }}>
        {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
    </td>
{/if}
{#if meterSettings.dps}
    <td class="px-1 text-center">
        {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
    </td>
{/if}
{#if !isSolo && meterSettings.damagePercent}
    <td class="px-1 text-center">
        {damagePercentage}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.critRate}
    <td class="px-1 text-center">
        {critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.critDmg}
    <td class="px-1 text-center">
        {critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anyFrontAtk && meterSettings.frontAtk}
    <td class="px-1 text-center">
        {faPercentage}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anyBackAtk && meterSettings.backAtk}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportBuff && meterSettings.percentBuffBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBrand && meterSettings.percentBrand}
    <td class="px-1 text-center">
        {round((entity.damageStats.debuffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.counters}
    <td class="px-1 text-center">
        {entity.skillStats.counters}<span class="text-3xs text-gray-300" />
    </td>
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {HexToRgba(color, alpha)}; width: {width}%" />
