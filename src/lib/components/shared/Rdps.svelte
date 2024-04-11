<script lang="ts">
    import RdpsHeader from "$lib/components/shared/RdpsHeader.svelte";
    import { type Entity, EntityType } from "$lib/types";
    import RdpsRow from "$lib/components/shared/RdpsRow.svelte";
    import { getRDamage } from "$lib/utils/numbers";
    import { rdpsEventDetails, takingScreenshot } from "$lib/utils/stores";

    export let players: Array<Entity>;
    export let totalDamageDealt: number;
    export let duration: number;
    export let meterSettings: any;

    let sortedPlayers: Entity[];
    let topRDamage: number;
    let playerRDamagePercentages: number[];
    let alpha: number;
    $: {
        if (players.length > 0) {
            sortedPlayers = players
                .filter((p) => p.entityType == EntityType.PLAYER)
                .toSorted((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
            topRDamage = getRDamage(sortedPlayers[0].damageStats);
            playerRDamagePercentages = sortedPlayers.map((p) => (getRDamage(p.damageStats) / topRDamage) * 100);
        }

        if (meterSettings.showClassColors !== undefined && !meterSettings.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
        }
    }
</script>

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
                    {alpha} />
            {/each}
        {/if}
    </tbody>
</table>
