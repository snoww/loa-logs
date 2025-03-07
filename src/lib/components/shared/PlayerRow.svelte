<script lang="ts">
    import { EntityType, type Entity, type IncapacitatedEvent } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { abbreviateNumberSplit, getBaseDamage, round } from "$lib/utils/numbers";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName, getEstherFromNpcId } from "$lib/utils/strings";
    import { generateArkPassiveTooltip, generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { localPlayer } from "$lib/utils/stores";

    interface Props {
        entity: Entity;
        totalDamageDealt: number;
        anyDead: boolean;
        multipleDeaths: boolean;
        anyFrontAtk: boolean;
        anyBackAtk: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        anySupportHat: boolean;
        anyRdpsData: boolean;
        anyPlayerIncapacitated: boolean;
        end: number;
        dps: (string | number)[];
        dpsRaw: number;
        alpha?: number;
        width: number;
        meterSettings: any;
        shadow?: boolean;
        isSolo: boolean;
    }

    let {
        entity,
        totalDamageDealt,
        anyDead,
        multipleDeaths,
        anyFrontAtk,
        anyBackAtk,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        anySupportHat,
        anyRdpsData,
        anyPlayerIncapacitated,
        end,
        dps,
        dpsRaw,
        alpha = 0.6,
        width,
        meterSettings,
        shadow = false,
        isSolo
    }: Props = $props();

    let damageDealtRaw: number = $derived(entity.damageStats.damageDealt);
    let damageDealt: (string | number)[] = $derived(abbreviateNumberSplit(damageDealtRaw));
    let damageWithoutHa: number = $derived(damageDealtRaw - (entity.damageStats.hyperAwakeningDamage ?? 0));
    let damagePercentage: string = $derived(((damageDealtRaw / totalDamageDealt) * 100).toFixed(1));
    let name: string = $state("");
    let tooltipName: string = $state("");
    let color = $state("#ffffff");
    let deadFor: string = $state("");

    let baseDamage = $derived(getBaseDamage(entity.damageStats));
    let sSynPercentage = $derived(((entity.damageStats.rdpsDamageReceivedSupport / baseDamage) * 100).toFixed(1));

    let critPercentage = $state("0.0");
    let critDmgPercentage = $state("0.0");
    let baPercentage = $state("0.0");
    let faPercentage = $state("0.0");

    $effect(() => {
        if (entity.entityType === EntityType.ESTHER) {
            name = getEstherFromNpcId(entity.npcId);
            tooltipName = name;
            color = "#4dc8d0";
        } else {
            name = formatPlayerName(entity, $settings.general);
            if ($settings.general.showNames) {
                tooltipName = entity.name;
            } else {
                tooltipName = entity.class;
            }
        }
    });

    $effect(() => {
        if (entity.skillStats.hits !== 0) {
            critDmgPercentage = round((entity.damageStats.critDamage / damageDealtRaw) * 100);
            critPercentage = round((entity.skillStats.crits / entity.skillStats.hits) * 100);
            if (
                meterSettings.positionalDmgPercent &&
                (entity.damageStats.frontAttackDamage > 0 || entity.damageStats.backAttackDamage > 0)
            ) {
                faPercentage = round((entity.damageStats.frontAttackDamage / damageDealtRaw) * 100);
                baPercentage = round((entity.damageStats.backAttackDamage / damageDealtRaw) * 100);
            } else {
                faPercentage = round((entity.skillStats.frontAttacks / entity.skillStats.hits) * 100);
                baPercentage = round((entity.skillStats.backAttacks / entity.skillStats.hits) * 100);
            }
        }
    });

    $effect(() => {
        if (entity.isDead) {
            deadFor = Math.abs((end - entity.damageStats.deathTime) / 1000).toFixed(0) + "s";
        }
    });

    $effect(() => {
        if (Object.hasOwn($colors, entity.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == entity.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[entity.class].color;
            }
        }
    });

    // compute total sum of time spent incapacitated for given events, accounting for overlap
    function computeIncapacitatedTime(events: IncapacitatedEvent[]) {
        if (!events.length) return 0;

        let totalTimeIncapacitated = 0;

        function addInterval(ivStart: number, ivEnd: number) {
            // clamp interval to the most recent damage event, such that
            // we don't count time spent incapacitated that has yet to happen
            ivStart = Math.min(ivStart, end);
            ivEnd = Math.min(ivEnd, end);
            totalTimeIncapacitated += Math.max(0, ivEnd - ivStart);
        }

        // collapse concurrent events so that we don't count the same time twice
        // note that the events array is guaranteed to be sorted by start time
        let curIntervalStart = events[0].timestamp;
        let curIntervalEnd = events[0].timestamp + events[0].duration;
        for (let i = 1; i < events.length; i++) {
            const event = events[i];

            // if this event starts after the current interval ends, add the current interval to the total
            if (event.timestamp > curIntervalEnd) {
                addInterval(curIntervalStart, curIntervalEnd);
                curIntervalStart = event.timestamp;
                curIntervalEnd = event.timestamp + event.duration;
            } else {
                // otherwise, extend the current interval
                curIntervalEnd = Math.max(curIntervalEnd, event.timestamp + event.duration);
            }
        }

        // add the last interval to the total
        addInterval(curIntervalStart, curIntervalEnd);
        return totalTimeIncapacitated;
    }

    const incapacitatedTimeMs = $derived.by(() => {
        const events = entity.damageStats.incapacitations;
        return {
            total: computeIncapacitatedTime(events),
            knockDown: computeIncapacitatedTime(events.filter((event) => event.type === "FALL_DOWN")),
            cc: computeIncapacitatedTime(events.filter((event) => event.type === "CROWD_CONTROL"))
        };
    });

    const incapacitationTooltip = $derived.by(() => {
        const { knockDown, cc } = incapacitatedTimeMs;
        return `<div class="font-normal text-xs flex flex-col space-y-1 -mx-px py-px">
            <span class="3xs text-gray-300">Knockdowns: ${(knockDown / 1000).toFixed(1)}s</span>
            <span class="3xs text-gray-300">Crowd control: ${(cc / 1000).toFixed(1)}s</span>
        </div>`;
    });
</script>

<td class="pl-1">
    {#if $settings.general.showEsther && entity.entityType === EntityType.ESTHER}
        <img class="table-cell size-5" src={$classIconCache[name]} alt={name} use:tooltip={{ content: name }} />
    {:else}
        <img
            class="table-cell size-5"
            src={$classIconCache[entity.classId]}
            alt={entity.class}
            use:tooltip={{ content: generateClassTooltip(entity) }} />
    {/if}
</td>
<td colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: generateArkPassiveTooltip(name, entity.arkPassiveData, entity.spec) }}>
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
{#if multipleDeaths && meterSettings.deathTime}
    <td class="px-1 text-center">
        {#if entity.damageStats.deaths > 0}
            {entity.damageStats.deaths}
        {:else}
            -
        {/if}
    </td>
{/if}
{#if anyPlayerIncapacitated && meterSettings.incapacitatedTime}
    <td class="px-1 text-center">
        {#if incapacitatedTimeMs.total > 0}
            <span use:tooltip={{ content: incapacitationTooltip }}>
                {(incapacitatedTimeMs.total / 1000).toFixed(1)}s
            </span>
        {/if}
    </td>
{/if}
{#if meterSettings.damage}
    <td class="px-1 text-center" use:tooltip={{ content: damageDealtRaw.toLocaleString() }}>
        {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
    </td>
{/if}
{#if meterSettings.dps}
    <td class="px-1 text-center" use:tooltip={{ content: dpsRaw.toLocaleString() }}>
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
        {faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anyBackAtk && meterSettings.backAtk}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBuff && meterSettings.percentBuffBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedBySupport / damageWithoutHa) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportBrand && meterSettings.percentBrand}
    <td class="px-1 text-center">
        {round((entity.damageStats.debuffedBySupport / damageWithoutHa) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportIdentity && meterSettings.percentIdentityBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedByIdentity / damageWithoutHa) * 100)}<span class="text-3xs text-gray-300"
            >%</span>
    </td>
{/if}
{#if anySupportHat && meterSettings.percentHatBySup}
    <td class="px-1 text-center">
        {round(((entity.damageStats.buffedByHat ?? 0) / damageDealtRaw) * 100)}<span class="text-3xs text-gray-300"
    >%</span>
    </td>
{/if}
{#if anyRdpsData && meterSettings.ssyn}
    <td
        class="px-1 text-center"
        use:tooltip={{
            content: `<span class="italic">${tooltipName}</span> dealt +${sSynPercentage}% more damage from support buffs`
        }}>
        {sSynPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings.counters}
    <td class="px-1 text-center">
        {entity.skillStats.counters}<span class="text-3xs text-gray-300"></span>
    </td>
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {HexToRgba(color, alpha)}; width: {width}%"></td>
