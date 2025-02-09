<script lang="ts">
    import { EntityType, type Entity } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";

    import DamageTakenRow from "./DamageTakenRow.svelte";

    interface Props {
        players: Array<Entity>;
        topDamageTaken: number | undefined;
        tween?: boolean;
    }

    let { players, topDamageTaken, tween = true }: Props = $props();

    let playerDamageTakenPercentages: Array<number> = $state([]);
    let alpha = $state(0.6);
    let sortedPlayers: Array<Entity> = $state(
        players
            .filter((e) => e.damageStats.damageTaken > 0 && e.entityType === EntityType.PLAYER)
            .toSorted((a, b) => b.damageStats.damageTaken - a.damageStats.damageTaken)
    );

    $effect(() => {
        if (topDamageTaken) {
            playerDamageTakenPercentages = sortedPlayers.map(
                (player) => (player.damageStats.damageTaken / topDamageTaken!) * 100
            );
        }
    });

    $effect.pre(() => {
        if (!$settings.meter.showClassColors) {
            alpha = 0;
        } else {
            alpha = 0.6;
        }
    });
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tight">
            <th class="w-7 px-2 font-normal"></th>
            <th class="w-14 px-2 text-left font-normal"></th>
            <th class="w-full"></th>
            <th class="w-28 font-normal" use:tooltip={{ content: "Total Damage Taken" }}>Damage Taken</th>
        </tr>
    </thead>
    <tbody class="relative z-10">
        {#each sortedPlayers as player, i (player.name)}
            <tr
                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                animate:flip={{ duration: 200 }}>
                <DamageTakenRow {player} {alpha} width={playerDamageTakenPercentages[i]} {tween} />
            </tr>
        {/each}
    </tbody>
</table>
