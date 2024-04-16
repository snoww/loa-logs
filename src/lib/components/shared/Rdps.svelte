<script lang="ts">
    import RdpsHeader from "$lib/components/shared/RdpsHeader.svelte";
    import { type Entity, EntityType, type PartyInfo } from "$lib/types";
    import RdpsRow from "$lib/components/shared/RdpsRow.svelte";
    import { getRDamage } from "$lib/utils/numbers";
    import { rdpsEventDetails, takingScreenshot } from "$lib/utils/stores";

    export let players: Array<Entity>;
    export let totalDamageDealt: number;
    export let duration: number;
    export let meterSettings: any;
    export let encounterPartyInfo: PartyInfo | undefined;

    let sortedPlayers: Entity[] = [];
    let topRDamage: number;
    let playerRDamagePercentages: number[];
    let alpha: number;
    let partySortedPlayers: Array<Array<Entity>> = [];
    let partyRDamgePercentages: number[][];
    let isLiveMeter = meterSettings.bossHp !== undefined;
    $: {
        if (players.length > 0) {
            sortedPlayers = players
                .filter((p) => p.entityType == EntityType.PLAYER)
                .toSorted((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
            topRDamage = getRDamage(sortedPlayers[0].damageStats);
            playerRDamagePercentages = sortedPlayers.map((p) => (getRDamage(p.damageStats) / topRDamage) * 100);

            if (meterSettings.rdpsSplitParty && encounterPartyInfo) {
                const parties = new Array<Array<Entity>>();
                const partyInfo = Object.entries(encounterPartyInfo);
                const partyPercentages = new Array<number[]>();
                if (partyInfo.length >= 2) {
                    for (const [partyIdStr, names] of partyInfo) {
                        const partyId = Number(partyIdStr);
                        parties[partyId] = [];
                        for (const name of names) {
                            const player = players.find((player) => player.name === name);
                            if (player) {
                                parties[partyId].push(player);
                            }
                        }
                        if (parties[partyId] && parties[partyId].length > 0) {
                            parties[partyId].sort((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
                            partyPercentages[partyId] = parties[partyId].map(
                                (player) => (getRDamage(player.damageStats) / topRDamage) * 100
                            );
                        }
                    }
                } else {
                    parties[0] = players;
                }

                partySortedPlayers = parties;
                partyRDamgePercentages = partyPercentages;
            }
        }

        if (meterSettings.showClassColors !== undefined && !meterSettings.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
        }
    }
</script>

{#if players.length > 0 && $rdpsEventDetails === "" && meterSettings.rdpsSplitParty && encounterPartyInfo && partySortedPlayers.length > 1}
    <div class="flex flex-col" class:space-y-2={!isLiveMeter}>
        {#each partySortedPlayers as partyMember, i (i)}
            <table class="relative w-full table-fixed">
                <RdpsHeader {meterSettings} partyId={i} />
                <tbody class="relative z-10">
                    {#each partyMember as player, j (player.name)}
                        <RdpsRow
                            {meterSettings}
                            {player}
                            width={partyRDamgePercentages[i][j]}
                            shadow={!$takingScreenshot}
                            {totalDamageDealt}
                            {duration}
                            {alpha}
                            {isLiveMeter} />
                    {/each}
                </tbody>
            </table>
        {/each}
    </div>
{:else}
    <table class="relative w-full table-fixed">
        <RdpsHeader {meterSettings} />
        <tbody class="relative z-10">
            {#if players.length > 0 && $rdpsEventDetails === ""}
                {#each sortedPlayers as player, i (player.name)}
                    <RdpsRow
                        {meterSettings}
                        {player}
                        width={playerRDamagePercentages[i]}
                        shadow={!$takingScreenshot}
                        {totalDamageDealt}
                        {duration}
                        {alpha}
                        {isLiveMeter} />
                {/each}
            {/if}
        </tbody>
    </table>
{/if}
