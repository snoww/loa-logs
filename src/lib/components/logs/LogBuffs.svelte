<script lang="ts">
    import {
        type EncounterDamageStats,
        type Entity,
        MeterTab,
        type StatusEffect,
        EntityType,
        BuffDetails
    } from "$lib/types";
    import { calculatePartyWidth, filterStatusEffects, getPartyBuffs } from "$lib/utils/buffs";
    import LogPartyBuffRow from "./LogPartyBuffRow.svelte";
    import LogBuffBreakdown from "./LogBuffBreakdown.svelte";
    import { settings } from "$lib/utils/settings";
    import LogBuffRow from "./LogBuffRow.svelte";
    import BuffHeader from "../shared/BuffHeader.svelte";

    let {
        tab,
        encounterDamageStats,
        players,
        focusedPlayer = undefined,
        inspectPlayer
    }: {
        tab: MeterTab;
        encounterDamageStats: EncounterDamageStats;
        players: Array<Entity>;
        focusedPlayer?: Entity;
        inspectPlayer: (name: string) => void;
    } = $props();

    if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
        focusedPlayer = undefined;
    }

    players = players.filter((player) => player.entityType === EntityType.PLAYER);
    let percentages = players.map(
        (player) => (player.damageStats.damageDealt / encounterDamageStats.topDamageDealt) * 100
    );

    let groupedSynergies = $state(new Map<string, Map<number, StatusEffect>>());
    $effect(() => {
        for (const [id, buff] of Object.entries(encounterDamageStats.buffs)) {
            if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
                continue;
            }
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer, tab, $settings.buffs.default);
        }
        for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
            if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
                continue;
            }
            filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer, tab, $settings.buffs.default);
        }
        groupedSynergies = new Map([...groupedSynergies.entries()].sort());
    });

    let parties = $state<Entity[][]>([]);
    let partyGroupedSynergies = $state(new Map<string, Set<string>>());
    let partyPercentages = $state<number[][]>([]);

    let partyBuffs = $state(new Map<string, Map<string, BuffDetails[]>>());

    let vw = $state(0);
    let partyWidths = $state<{ [key: string]: string }>({});

    $effect(() => {
        if ($settings.logs.splitPartyBuffs && encounterDamageStats.misc?.partyInfo && !focusedPlayer) {
            const partyBuffsObj = getPartyBuffs(
                players,
                encounterDamageStats.topDamageDealt,
                encounterDamageStats.misc.partyInfo,
                groupedSynergies
            );
            parties = partyBuffsObj.parties;
            partyGroupedSynergies = partyBuffsObj.partyGroupedSynergies;
            partyPercentages = partyBuffsObj.partyPercentages;
            partyBuffs = partyBuffsObj.partyBuffs;
        }
    });

    $effect(() => {
        if (partyGroupedSynergies.size > 0) {
            const remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
            partyWidths = calculatePartyWidth(partyGroupedSynergies, remToPx, vw);
        }
    });
</script>

<svelte:window bind:innerWidth={vw} />
{#if $settings.logs.splitPartyBuffs && parties.length > 1 && tab === MeterTab.PARTY_BUFFS && !focusedPlayer}
    <div class="flex flex-col space-y-2">
        {#each [...partyGroupedSynergies] as [partyId, synergies], i (partyId)}
            {#if parties[i] && parties[i].length > 0}
                <table class="table-fixed" style="width: {partyWidths[partyId]};">
                    <thead class="z-40 h-6" id="buff-head">
                        <tr class="bg-zinc-900">
                            <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Party {+partyId + 1}</th>
                            <th class="w-20 px-2 text-left font-normal"></th>
                            <th class="w-full"></th>
                            {#each [...synergies] as synergy (synergy)}
                                {@const syns = groupedSynergies.get(synergy) || new Map()}
                                <BuffHeader synergies={syns} />
                            {/each}
                        </tr>
                    </thead>
                    <tbody class="relative z-10">
                        {#each parties[i] as player, playerIndex (player.name)}
                            {@const playerBuffs = partyBuffs.get(partyId)?.get(player.name) ?? []}
                            <tr
                                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                                onclick={() => inspectPlayer(player.name)}>
                                <LogPartyBuffRow {player} {playerBuffs} percentage={partyPercentages[i][playerIndex]} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        {/each}
    </div>
{:else}
    <table class="w-full table-fixed">
        <thead class="relative z-40 h-6" id="buff-head">
            <tr class="bg-zinc-900">
                <th class="w-7 px-2 font-normal"></th>
                <th class="w-20 px-2 text-left font-normal"></th>
                <th class="w-full"></th>
                {#each [...groupedSynergies] as [id, synergies] (id)}
                    <BuffHeader {synergies} />
                {:else}
                    <th class="font-normal w-20">No Buffs</th>
                {/each}
            </tr>
        </thead>
        <tbody class="relative z-10">
            {#if !focusedPlayer}
                {#each players as player, i (player.name)}
                    <tr
                        class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                        onclick={() => inspectPlayer(player.name)}>
                        <LogBuffRow {player} {groupedSynergies} percentage={percentages[i]} />
                    </tr>
                {/each}
            {:else}
                <LogBuffBreakdown {groupedSynergies} player={focusedPlayer} {tab} />
            {/if}
        </tbody>
    </table>
{/if}
