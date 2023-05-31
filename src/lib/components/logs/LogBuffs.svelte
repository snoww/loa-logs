<script lang="ts">
    import { type EncounterDamageStats, type Entity, MeterTab, type StatusEffect, EntityType } from "$lib/types";
    import { filterStatusEffects } from "$lib/utils/buffs";
    import LogBuffHeader from "./LogBuffHeader.svelte";
    import LogBuffRow from "./LogBuffRow.svelte";
    import LogBuffBreakdown from "./LogBuffBreakdown.svelte";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats;
    export let players: Array<Entity>;
    export let focusedPlayer: Entity | null = null;
    export let handleRightClick: () => void;
    export let inspectPlayer: (name: string) => void;

    if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
        focusedPlayer = null;
        handleRightClick();
    }
    players = players.filter((player) => player.entityType === EntityType.PLAYER);
    let percentages = players.map(
        (player) => (player.damageStats.damageDealt / encounterDamageStats.topDamageDealt) * 100
    );
    let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
    for (const [id, buff] of Object.entries(encounterDamageStats.buffs)) {
        if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
            continue;
        }
        if (buff.category === "buff") {
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer, tab);
        }
    }
    for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
        if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
            continue;
        }
        if (debuff.category === "debuff") {
            filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer, tab);
        }
    }
    groupedSynergies = new Map([...groupedSynergies.entries()].sort());
</script>

<thead class="relative z-40 h-6" id="buff-head">
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal" />
        <th class="w-20 px-2 text-left font-normal" />
        <th class="w-full" />
        {#each [...groupedSynergies] as [id, synergies] (id)}
            <LogBuffHeader {synergies} />
        {:else}
            <th class="font-normal w-20">No Buffs</th>
        {/each}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick} class="relative z-10">
    {#if !focusedPlayer}
        {#each players as player, i (player.name)}
            <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
                <LogBuffRow {player} {groupedSynergies} percentage={percentages[i]} />
            </tr>
        {/each}
    {:else}
        <LogBuffBreakdown {groupedSynergies} player={focusedPlayer} />
    {/if}
</tbody>
