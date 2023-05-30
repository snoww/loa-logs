<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { EntityType, type Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";

    export let entity: Entity;
    export let percentage: number;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportBrand: boolean;
    export let end: number;

    let damageDealt: (string | number)[];
    let dps: (string | number)[];
    let damagePercentage: string;
    let damagePercentageRaw: number;
    let name: string;
    let deadFor: string;

    let color = "#ffffff";

    if (Object.hasOwn(classColors, entity.class)) {
        color = classColors[entity.class].color;
    }
    damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    damagePercentageRaw = (entity.damageStats.damageDealt / totalDamageDealt) * 100;
    damagePercentage = damagePercentageRaw.toFixed(1);

    dps = abbreviateNumberSplit(entity.damageStats.dps);

    $: {
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general.showNames, $settings.general.showGearScore);
        }
    }
    if (entity.isDead) {
        deadFor = (((end - entity.damageStats.deathTime) / 1000).toFixed(0) + "s").replace("-", "");
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
    <div class="truncate" use:tooltip={{ content: name }}>
        {name}
    </div>
</td>
{#if anyDead && $settings.logs.deathTime}
    <td class="px-1 text-center">
        {#if entity.isDead}
            {deadFor}
        {/if}
    </td>
{/if}
{#if $settings.logs.damage}
    <td class="px-1 text-center">
        {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
    </td>
{/if}
{#if $settings.logs.dps}
    <td class="px-1 text-center">
        {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
    </td>
{/if}
{#if damagePercentageRaw < 100 && $settings.logs.damagePercent}
    <td class="px-1 text-center">
        {damagePercentage}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if $settings.logs.critRate}
    <td class="px-1 text-center">
        {round((entity.skillStats.crits / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anyFrontAtk && $settings.logs.frontAtk}
    <td class="px-1 text-center">
        {round((entity.skillStats.frontAttacks / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anyBackAtk && $settings.logs.backAtk}
    <td class="px-1 text-center">
        {round((entity.skillStats.backAttacks / entity.skillStats.hits) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportBuff && $settings.logs.percentBuffBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBrand && $settings.logs.percentBrand}
    <td class="px-1 text-center">
        {round((entity.damageStats.debuffedBySupport / entity.damageStats.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if $settings.logs.counters}
    <td class="px-1 text-center">
        {entity.skillStats.counters}<span class="text-3xs text-gray-300" />
    </td>
{/if}
<div
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%" />
