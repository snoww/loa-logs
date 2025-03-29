<script lang="ts">
    import { EntityType, type Entity } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { round } from "$lib/utils/numbers";
    import { classIconCache, settings, UWUOWO_URL } from "$lib/utils/settings";
    import { generateArkPassiveTooltip, generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { isValidName } from "$lib/utils/strings";
    import { open } from "@tauri-apps/api/shell";

    interface Props {
        enc: EncounterState;
        entity: Entity;
        width: number;
        shadow?: boolean;
        anyDead?: boolean;
    }

    let { enc, entity, width, shadow = false, anyDead = undefined }: Props = $props();

    let entityState = $derived(new EntityState(entity, enc));

    let alpha = $derived(enc.live && !enc.curSettings.showClassColors ? 0 : 0.6);

    const incapacitationTooltip = $derived.by(() => {
        const { knockDown, cc } = entityState.incapacitatedTimeMs;
        return `<div class="font-normal text-xs flex flex-col space-y-1 -mx-px py-px">
            <span class="3xs text-gray-300">Knockdowns: ${(knockDown / 1000).toFixed(1)}s</span>
            <span class="3xs text-gray-300">Crowd control: ${(cc / 1000).toFixed(1)}s</span>
        </div>`;
    });

    let tweenedValue = new Tween(enc.live ? 0 : width, {
        duration: 400,
        easing: cubicOut
    });
    $effect(() => {
        tweenedValue.set(width ?? 0);
    });

    let hovering = $state(false);
</script>

<td class="pl-1">
    {#if $settings.general.showEsther && entity.entityType === EntityType.ESTHER}
        <img
            class="table-cell size-5"
            src={$classIconCache[entityState.name]}
            alt={entityState.name}
            use:tooltip={{ content: entityState.name }} />
    {:else}
        <img
            class="table-cell size-5"
            src={$classIconCache[entity.classId]}
            alt={entity.class}
            use:tooltip={{ content: generateClassTooltip(entity) }} />
    {/if}
</td>
<td colspan="2" onmouseenter={() => (hovering = true)} onmouseleave={() => (hovering = false)}>
    <div class="flex gap-1">
        <div class="truncate">
            <span
                use:tooltip={{
                    content: generateArkPassiveTooltip(entityState.name, entity.arkPassiveData, entity.spec)
                }}>
                {entityState.name}
            </span>
        </div>
        {#if !enc.live && isValidName(entityState.entity.name) && hovering && entityState.entity.entityType === EntityType.PLAYER}
            <span>
                <button
                    class="flex-1 tracking-tighter hover:underline"
                    onclick={(e) => {
                        e.stopPropagation();
                        open(UWUOWO_URL + "/character/" + enc.region + "/" + entityState.entity.name);
                    }}>uwu</button>
            </span>
        {/if}
    </div>
</td>
{#if anyDead !== undefined ? anyDead : enc.anyDead && enc.curSettings.deathTime}
    <td class="px-1 text-center">
        {#if entity.isDead}
            {entityState.deadFor}
        {/if}
    </td>
{/if}
{#if enc.multipleDeaths && enc.curSettings.deathTime}
    <td class="px-1 text-center">
        {#if entity.damageStats.deaths > 0}
            {entity.damageStats.deaths}
        {:else}
            -
        {/if}
    </td>
{/if}
{#if enc.anyPlayerIncapacitated && enc.curSettings.incapacitatedTime}
    <td class="px-1 text-center">
        {#if entityState.incapacitatedTimeMs.total > 0}
            <span use:tooltip={{ content: incapacitationTooltip }}>
                {(entityState.incapacitatedTimeMs.total / 1000).toFixed(1)}s
            </span>
        {/if}
    </td>
{/if}
{#if enc.curSettings.damage}
    <td class="px-1 text-center" use:tooltip={{ content: entityState.damageDealt.toLocaleString() }}>
        {entityState.damageDealtString[0]}<span class="text-3xs text-gray-300">{entityState.damageDealtString[1]}</span>
    </td>
{/if}
{#if enc.curSettings.dps}
    <td class="px-1 text-center" use:tooltip={{ content: entityState.dps.toLocaleString() }}>
        {entityState.dpsString[0]}<span class="text-3xs text-gray-300">{entityState.dpsString[1]}</span>
    </td>
{/if}
{#if !enc.isSolo && enc.curSettings.damagePercent}
    <td class="px-1 text-center">
        {entityState.damagePercentage}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.curSettings.critRate}
    <td class="px-1 text-center">
        {entityState.critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.curSettings.critDmg}
    <td class="px-1 text-center">
        {entityState.critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anyFrontAtk && enc.curSettings.frontAtk}
    <td class="px-1 text-center">
        {entityState.faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anyBackAtk && enc.curSettings.backAtk}
    <td class="px-1 text-center">
        {entityState.baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anySupportBuff && enc.curSettings.percentBuffBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedBySupport / entityState.damageDealtWithoutHa) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anySupportBrand && enc.curSettings.percentBrand}
    <td class="px-1 text-center">
        {round((entity.damageStats.debuffedBySupport / entityState.damageDealtWithoutHa) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anySupportIdentity && enc.curSettings.percentIdentityBySup}
    <td class="px-1 text-center">
        {round((entity.damageStats.buffedByIdentity / entityState.damageDealtWithoutHa) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.anySupportHat && enc.curSettings.percentHatBySup}
    <td class="px-1 text-center">
        {round(((entity.damageStats.buffedByHat ?? 0) / entityState.damageDealt) * 100)}<span
            class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if enc.curSettings.counters}
    <td class="px-1 text-center">
        {entity.skillStats.counters}<span class="text-3xs text-gray-300"></span>
    </td>
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {HexToRgba(entityState.color, alpha)}; width: {tweenedValue.current}%"></td>
