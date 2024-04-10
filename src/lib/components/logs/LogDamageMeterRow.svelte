<script lang="ts">
    import type { Entity } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import PlayerRow from "../shared/PlayerRow.svelte";

    export let entity: Entity;
    export let percentage: number;
    export let totalDamageDealt: number;
    export let anyDead: boolean;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportIdentity: boolean;
    export let anySupportBrand: boolean;
    export let anyRdpsData: boolean;
    export let end: number;
    export let isSolo: boolean;

    let dps = abbreviateNumberSplit(entity.damageStats.dps);
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
    {anyRdpsData}
    {end}
    {dps}
    width={percentage}
    meterSettings={$settings.logs}
    shadow={!$takingScreenshot}
    {isSolo} />
