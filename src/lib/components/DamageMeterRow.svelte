<script lang="ts">
    import type { Entity } from "$lib/types";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import PlayerRow from "./shared/PlayerRow.svelte";

    interface Props {
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
    }

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
    }: Props = $props();

    let alpha = $state(0.6);

    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });
    $effect(() => {
        tweenedValue.set(percentage ?? 0);
    });

    let dps: (string | number)[] = $state([]);
    let dpsRaw = $derived(Math.round(entity.damageStats.damageDealt / (duration / 1000)));

    $effect(() => {
        if (duration > 0) {
            dps = abbreviateNumberSplit(dpsRaw);
        } else {
            dps = ["0", ""];
        }
    });

    $effect(() => {
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
    {dpsRaw}
    end={lastCombatPacket}
    {dps}
    {alpha}
    width={tweenedValue.current}
    meterSettings={$settings.meter}
    {isSolo} />
