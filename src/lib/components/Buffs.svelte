<script lang="ts">
    import { MeterTab, type EncounterDamageStats, type Entity, type StatusEffect, EntityType } from "$lib/types";
    import { filterStatusEffects } from "$lib/utils/buffs";
    import { flip } from "svelte/animate";
    import BuffHeader from "./BuffHeader.svelte";
    import BuffRow from "./BuffRow.svelte";
    import BuffSkillBreakdown from "./BuffSkillBreakdown.svelte";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats | undefined;
    export let players: Array<Entity>;
    export let focusedPlayer: Entity | null = null;
    export let handleRightClick: () => void;
    export let inspectPlayer: (name: string) => void;

    let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
    let percentages = Array<number>();

    $: {
        if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
            focusedPlayer = null;
            handleRightClick();
        }
        players = players.filter((player) => player.entityType === EntityType.PLAYER);
        groupedSynergies = new Map<string, Map<number, StatusEffect>>();
        if (encounterDamageStats) {
            percentages = players.map(
                (player) => (player.damageStats.damageDealt / encounterDamageStats!.topDamageDealt) * 100
            );
            Object.entries(encounterDamageStats.buffs).forEach(([id, buff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
                    return;
                }
                filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer, tab);
            });
            Object.entries(encounterDamageStats.debuffs).forEach(([id, debuff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
                    return;
                }
                filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer, tab);
            });
            groupedSynergies = new Map([...groupedSynergies.entries()].sort());
        }
    }
</script>

<thead class="sticky top-0 z-40 h-6">
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal" />
        <th class="w-14 px-2 text-left font-normal" />
        <th class="w-full" />
        {#each [...groupedSynergies] as [id, synergies] (id)}
            <BuffHeader {synergies} />
        {:else}
            <th class="font-normal w-20">No Buffs</th>
        {/each}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#if !focusedPlayer}
        {#each players as player, i (player.id)}
            <tr class="h-7 px-2 py-1" animate:flip={{ duration: 200 }} on:click={() => inspectPlayer(player.name)}>
                <BuffRow {player} {groupedSynergies} percentage={percentages[i]} />
            </tr>
        {/each}
    {:else}
        <BuffSkillBreakdown {groupedSynergies} player={focusedPlayer} />
    {/if}
</tbody>
