<script lang="ts">
    import RdpsHeader from "$lib/components/shared/RdpsHeader.svelte";
    import { type Entity, EntityType } from "$lib/types";
    import RdpsRow from "$lib/components/shared/RdpsRow.svelte";
    import { getRDamage } from "$lib/utils/numbers";
    import { takingScreenshot } from "$lib/utils/stores";

    export let players: Array<Entity>;
    export let totalDamageDealt: number;
    export let duration: number;

    let sortedPlayers: Entity[];
    let topRDamage: number;
    let playerRDamagePercentages: number[];
    $: {
        if (players.length > 0) {
            sortedPlayers = players
                .filter((p) => p.entityType == EntityType.PLAYER)
                .toSorted((a, b) => getRDamage(b.damageStats) - getRDamage(a.damageStats));
            topRDamage = getRDamage(sortedPlayers[0].damageStats);
            playerRDamagePercentages = sortedPlayers.map((p) => (getRDamage(p.damageStats) / topRDamage) * 100);
        }
    }
</script>

<table class="relative w-full table-fixed">
    <RdpsHeader />
    <tbody class="relative z-10">
        {#if players.length > 0}
            {#each sortedPlayers as player, i (player.name)}
                <RdpsRow
                    {player}
                    width={playerRDamagePercentages[i]}
                    shadow={!$takingScreenshot}
                    {totalDamageDealt}
                    {duration} />
            {/each}
        {/if}
    </tbody>
</table>
