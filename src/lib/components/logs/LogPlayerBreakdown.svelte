<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { classIconCache, colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "../shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";
    import { generateArkPassiveTooltip, generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { formatPlayerName } from "$lib/utils/strings";
    import { localPlayer, takingScreenshot } from "$lib/utils/stores";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import PlayerBreakdownRow from "../shared/PlayerBreakdownRow.svelte";

    interface Props {
        entity: Entity;
        enc: EncounterState;
    }

    let { entity, enc }: Props = $props();
    let entityState = $derived(new EntityState(entity, enc));
</script>

<thead class="z-30 h-6">
    <tr class="bg-zinc-900">
        <PlayerBreakdownHeader {entityState} />
    </tr>
</thead>
<tbody class="relative z-10">
    {#if entity.entityType !== EntityType.ESTHER}
        <tr class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
            <td class="pl-1">
                <img
                    class="table-cell size-5"
                    src={$classIconCache[entity.classId]}
                    alt={entity.class}
                    use:tooltip={{ content: generateClassTooltip(entity) }} />
            </td>
            <td colspan="2">
                <div class="truncate">
                    <span use:tooltip={{ content: generateArkPassiveTooltip(entityState.name, entity.arkPassiveData, entity.spec) }}>
                        {entityState.name}
                    </span>
                </div>
            </td>
            {#if $settings.logs.breakdown.damage}
                <td class="px-1 text-center" use:tooltip={{ content: entity.damageStats.damageDealt.toLocaleString() }}>
                    {entityState.damageDealtString[0]}<span class="text-3xs text-gray-300"
                        >{entityState.damageDealtString[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.dps}
                <td class="px-1 text-center" use:tooltip={{ content: entity.damageStats.dps.toLocaleString() }}>
                    {entityState.dpsString[0]}<span class="text-3xs text-gray-300">{entityState.dpsString[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.damagePercent}
                <td class="px-1 text-center">
                    {entityState.damagePercentage}<span class="text-xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.critRate}
                <td class="px-1 text-center">
                    {entityState.critPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.adjustedCritRate}
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.critDmg}
                <td class="px-1 text-center">
                    {entityState.critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anyFrontAttacks && $settings.logs.breakdown.frontAtk}
                <td class="px-1 text-center">
                    {entityState.faPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anyBackAttacks && $settings.logs.breakdown.backAtk}
                <td class="px-1 text-center">
                    {entityState.baPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anySupportBuff && $settings.logs.breakdown.percentBuffBySup}
                <td class="px-1 text-center">
                    {round((entity.damageStats.buffedBySupport / entityState.damageDealtWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anySupportBrand && $settings.logs.breakdown.percentBrand}
                <td class="px-1 text-center">
                    {round((entity.damageStats.debuffedBySupport / entityState.damageDealtWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anySupportIdentity && $settings.logs.breakdown.percentIdentityBySup}
                <td class="px-1 text-center">
                    {round((entity.damageStats.buffedByIdentity / entityState.damageDealtWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if entityState.anySupportHat && $settings.logs.breakdown.percentHatBySup}
                <td class="px-1 text-center">
                    {round(((entity.damageStats.buffedByHat ?? 0) / entity.damageStats.damageDealt) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.avgDamage}
                <td class="px-1 text-center"> - </td>
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.maxDamage}
                <td class="px-1 text-center"> - </td>
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.casts}
                <td
                    class="px-1 text-center"
                    use:tooltip={{
                        content: `<div class="py-1">${
                            entity.skillStats.casts.toLocaleString() +
                            " " +
                            (entity.skillStats.casts === 1 ? "cast" : "casts")
                        }</div>`
                    }}>
                    {abbreviateNumberSplit(entity.skillStats.casts)[0]}<span class="text-3xs text-gray-300"
                        >{abbreviateNumberSplit(entity.skillStats.casts)[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.cpm}
                <td class="px-1 text-center">
                    <div
                        use:tooltip={{
                            content: `<div class="py-1">${
                                entity.skillStats.casts.toLocaleString() +
                                " " +
                                (entity.skillStats.casts === 1 ? "cast" : "casts")
                            }</div>`
                        }}>
                        {round(entity.skillStats.casts / (entityState.enc.duration / 1000 / 60))}
                    </div>
                </td>
            {/if}
            {#if $settings.logs.breakdown.hits}
                <td
                    class="px-1 text-center"
                    use:tooltip={{
                        content: `<div class="py-1">${
                            entity.skillStats.hits.toLocaleString() +
                            " " +
                            (entity.skillStats.hits === 1 ? "hit" : "hits")
                        }</div>`
                    }}>
                    {abbreviateNumberSplit(entity.skillStats.hits)[0]}<span class="text-3xs text-gray-300"
                        >{abbreviateNumberSplit(entity.skillStats.hits)[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.hpm}
                <td class="px-1 text-center">
                    {#if entity.skillStats.hits === 0}
                        <div class="">0</div>
                    {:else}
                        <div
                            use:tooltip={{
                                content: `<div class="py-1">${
                                    entity.skillStats.hits.toLocaleString() +
                                    " " +
                                    (entity.skillStats.hits === 1 ? "hit" : "hits")
                                }</div>`
                            }}>
                            {round(entity.skillStats.hits / (entityState.enc.duration / 1000 / 60))}
                        </div>
                    {/if}
                </td>
            {/if}
            <td
                class="absolute left-0 -z-10 h-7 px-2 py-1"
                class:shadow-md={!$takingScreenshot}
                style="background-color: {$settings.general.splitLines
                    ? RGBLinearShade(HexToRgba(entityState.color, 0.6))
                    : HexToRgba(entityState.color, 0.6)}; width: 100%"></td>
        </tr>
    {/if}
    {#each entityState.skills as skill, i (skill.id)}
        <tr class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
            <PlayerBreakdownRow
                {skill}
                {entityState}
                shadow={!$takingScreenshot}
                width={entityState.skillDamagePercentages[i]}
                index={i} />
        </tr>
    {/each}
</tbody>
