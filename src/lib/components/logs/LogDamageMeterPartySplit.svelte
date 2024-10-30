<script lang="ts">
    import { EntityType, type Entity, type PartyInfo } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import LogDamageMeterHeader from "./LogDamageMeterHeader.svelte";
    import LogDamageMeterRow from "./LogDamageMeterRow.svelte";

    export let players: Array<Entity>;
    export let encounterPartyInfo: PartyInfo | undefined;
    export let topDamageDealt: number;
    export let totalDamageDealt: number;
    export let anyFrontAtk: boolean;
    export let anyBackAtk: boolean;
    export let anySupportBuff: boolean;
    export let anySupportIdentity: boolean;
    export let anySupportBrand: boolean;
    export let anyRdpsData: boolean;
    export let end: number;
    export let isSolo: boolean;
    export let inspectPlayer: (name: string) => void;

    let parties = new Array<Array<Entity>>();
    let partyPercentages = new Array<number[]>();
    let anyPartyDead = new Array<boolean>();
    let multipleDeaths = false;

    let esthers = new Array<Entity>();

    $: {
        if (encounterPartyInfo) {
            esthers = players.filter((player) => player.entityType === EntityType.ESTHER);
            const partyInfo = Object.entries(encounterPartyInfo);
            if (partyInfo.length >= 2) {
                for (const [partyIdStr, names] of partyInfo) {
                    const partyId = Number(partyIdStr);
                    parties[partyId] = [];
                    anyPartyDead[partyId] = false;
                    for (const name of names) {
                        const player = players.find((player) => player.name === name);
                        if (player) {
                            parties[partyId].push(player);
                            if (player.isDead) {
                                anyPartyDead[partyId] = true;
                                if (!multipleDeaths && player.damageStats.deaths > 1) {
                                    multipleDeaths = true;
                                }
                            } else {
                                if (!multipleDeaths && player.damageStats.deaths > 0) {
                                    multipleDeaths = true;
                                }
                            }
                        }
                    }
                    if (parties[partyId] && parties[partyId].length > 0) {
                        parties[partyId].sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                        partyPercentages[partyId] = parties[partyId].map(
                            (player) => (player.damageStats.damageDealt / topDamageDealt) * 100
                        );
                    }
                }
            } else {
                parties[0] = players;
            }
        }
    }
</script>

<div class="flex flex-col space-y-2">
    {#each parties as party, partyId}
        {#if party && party.length > 0}
            <table class="w-full table-fixed">
                <thead class="z-40 h-6">
                    <tr class="bg-zinc-900">
                        <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Party {+partyId + 1}</th>
                        <th class="w-20 px-2 text-left font-normal" />
                        <th class="w-full" />
                        <LogDamageMeterHeader
                            anyDead={anyPartyDead[partyId]}
                            {multipleDeaths}
                            {anyFrontAtk}
                            {anyBackAtk}
                            {anySupportBuff}
                            {anySupportIdentity}
                            {anySupportBrand}
                            {anyRdpsData}
                            {isSolo} />
                    </tr>
                </thead>
                <tbody class="relative z-10">
                    {#each party as player, playerIndex (player.name)}
                        <tr
                            class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                            on:click={() => inspectPlayer(player.name)}>
                            <LogDamageMeterRow
                                entity={player}
                                percentage={partyPercentages[partyId][playerIndex]}
                                {totalDamageDealt}
                                anyDead={anyPartyDead[partyId]}
                                {multipleDeaths}
                                {anyFrontAtk}
                                {anyBackAtk}
                                {anySupportBuff}
                                {anySupportIdentity}
                                {anySupportBrand}
                                {anyRdpsData}
                                {end}
                                {isSolo} />
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    {/each}
    {#if esthers.length > 0 && $settings.general.showEsther}
        <table class="w-full table-fixed">
            <thead class="z-40 h-6">
                <tr class="bg-zinc-900">
                    <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Esthers</th>
                    <th class="w-20 px-2 text-left font-normal" />
                    <th class="w-full" />
                    <LogDamageMeterHeader
                        anyDead={false}
                        multipleDeaths={false}
                        {anyFrontAtk}
                        {anyBackAtk}
                        {anySupportBuff}
                        {anySupportIdentity}
                        {anySupportBrand}
                        {anyRdpsData}
                        {isSolo} />
                </tr>
            </thead>
            <tbody class="relative z-10">
                {#each esthers as esther (esther.name)}
                    <tr
                        class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                        on:click={() => inspectPlayer(esther.name)}>
                        <LogDamageMeterRow
                            entity={esther}
                            percentage={(esther.damageStats.damageDealt / topDamageDealt) * 100}
                            {totalDamageDealt}
                            anyDead={false}
                            multipleDeaths={false}
                            {anyFrontAtk}
                            {anyBackAtk}
                            {anySupportBuff}
                            {anySupportIdentity}
                            {anySupportBrand}
                            {anyRdpsData}
                            {end}
                            {isSolo} />
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
