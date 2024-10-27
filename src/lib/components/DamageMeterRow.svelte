<script lang="ts">
    import type { Entity } from "$lib/types";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import PlayerRow from "./shared/PlayerRow.svelte";

    let {
        entity,
        percentage,
        duration,
        totalDamageDealt,
        lastCombatPacket,
        anyDead,
        multipleDeaths,
        anyFrontAtk,
        anyBackAtk,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        anyRdpsData,
        isSolo
    }: {
        entity: Entity;
        percentage: number;
        duration: number;
        totalDamageDealt: number;
        lastCombatPacket: number;
        anyDead: boolean;
        multipleDeaths: boolean;
        anyFrontAtk: boolean;
        anyBackAtk: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        anyRdpsData: boolean;
        isSolo: boolean;
    } = $props();

    let alpha = $state(0.6);

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let dps = $state<(string | number)[]>([]);

    $effect(() => {
        tweenedValue.set(percentage);

        if (duration > 0) {
            dps = abbreviateNumberSplit(entity.damageStats.damageDealt / (duration / 1000));
        } else {
            dps = ["0", ""];
        }

        if (!$settings.meter.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
        }
    });
</script>

<PlayerRow
    {entity}
    {totalDamageDealt}
    {anyDead}
    {multipleDeaths}
    {anyFrontAtk}
    {anyBackAtk}
    {anySupportBuff}
    {anySupportIdentity}
    {anySupportBrand}
    {anyRdpsData}
    end={lastCombatPacket}
    {dps}
    {alpha}
    width={$tweenedValue}
    meterSettings={$settings.meter}
    {isSolo} />
