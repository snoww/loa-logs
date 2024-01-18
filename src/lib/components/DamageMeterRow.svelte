<script lang="ts">
    import type { Entity } from "$lib/types";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import PlayerRow from "./shared/PlayerRow.svelte";

    export let entity: Entity;
    export let percentage: number;
    export let duration: number;
    export let totalDamageDealt: number;
    export let lastCombatPacket: number;
    export let anyDead: boolean;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportIdentity: boolean;
    export let anySupportBrand: boolean;
    export let isSolo: boolean;

    let alpha = 0.6;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    let dps: (string | number)[];

    $: {
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
    }
</script>

<PlayerRow
    {entity}
    {totalDamageDealt}
    {anyDead}
    {anyFrontAtk}
    {anyBackAtk}
    {anySupportBuff}
    {anySupportIdentity}
    {anySupportBrand}
    end={lastCombatPacket}
    {dps}
    {alpha}
    width={$tweenedValue}
    meterSettings={$settings.meter}
    {isSolo} />
