<script lang="ts">
    import type { Entity } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import PlayerRow from "../shared/PlayerRow.svelte";

    let {
        entity,
        percentage,
        totalDamageDealt,
        anyDead,
        multipleDeaths,
        anyFrontAtk,
        anyBackAtk,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        anyRdpsData,
        end,
        isSolo
    }: {
        entity: Entity;
        percentage: number;
        totalDamageDealt: number;
        anyDead: boolean;
        multipleDeaths: boolean;
        anyFrontAtk: boolean;
        anyBackAtk: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        anyRdpsData: boolean;
        end: number;
        isSolo: boolean;
    } = $props();

    let dps = abbreviateNumberSplit(entity.damageStats.dps);
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
    {end}
    {dps}
    width={percentage}
    meterSettings={$settings.logs}
    shadow={!$takingScreenshot}
    {isSolo} />
