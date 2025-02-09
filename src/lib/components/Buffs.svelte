<script lang="ts">
    import {
        MeterTab,
        type EncounterDamageStats,
        type Entity,
        type StatusEffect,
        EntityType,
        BuffDetails,
        type PartyInfo
    } from "$lib/types";
    import { filterStatusEffects, getPartyBuffs } from "$lib/utils/buffs";
    import { flip } from "svelte/animate";
    import BuffRow from "./BuffRow.svelte";
    import BuffSkillBreakdown from "./BuffSkillBreakdown.svelte";
    import { settings } from "$lib/utils/settings";
    import BuffHeader from "./shared/BuffHeader.svelte";
    import PartyBuffRow from "./PartyBuffRow.svelte";

    interface Props {
        tab: MeterTab;
        encounterDamageStats: EncounterDamageStats | undefined;
        entities: Array<Entity>;
        focusedPlayer?: Entity | null;
        handleRightClick: () => void;
        inspectPlayer: (name: string) => void;
        encounterPartyInfo: PartyInfo | undefined;
        localPlayer: string | undefined;
    }

    let {
        tab,
        encounterDamageStats,
        entities,
        focusedPlayer = $bindable(null),
        handleRightClick,
        inspectPlayer,
        encounterPartyInfo,
        localPlayer
    }: Props = $props();

    let players = $derived(entities.filter((entity) => entity.entityType === EntityType.PLAYER));
    let groupedSynergies: Map<string, Map<number, StatusEffect>> = $state(new Map());
    let percentages = $state(Array<number>());

    let parties = $state(new Array<Array<Entity>>());
    let partyGroupedSynergies = $state(new Array<[string, Set<string>]>());
    let partyPercentages = $state(new Array<number[]>());

    let partyBuffs = $state(new Map<string, Map<string, Array<BuffDetails>>>());

    let localPlayerInP1 = $state(true);

    $effect(() => {
        if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
            focusedPlayer = null;
            handleRightClick();
        }
    });

    $effect(() => {
        let tempGroupedSynergies = new Map<string, Map<number, StatusEffect>>();
        if (encounterDamageStats) {
            percentages = players.map(
                (player) => (player.damageStats.damageDealt / encounterDamageStats!.topDamageDealt) * 100
            );
            Object.entries(encounterDamageStats.buffs).forEach(([id, buff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
                    return;
                }
                filterStatusEffects(
                    tempGroupedSynergies,
                    buff,
                    Number(id),
                    focusedPlayer,
                    tab,
                    $settings.buffs.default
                );
            });
            Object.entries(encounterDamageStats.debuffs).forEach(([id, debuff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
                    return;
                }
                filterStatusEffects(
                    tempGroupedSynergies,
                    debuff,
                    Number(id),
                    focusedPlayer,
                    tab,
                    $settings.buffs.default
                );
            });
            tempGroupedSynergies = new Map([...tempGroupedSynergies.entries()].sort());
            if (
                $settings.meter.splitPartyBuffs &&
                encounterPartyInfo &&
                Object.keys(encounterPartyInfo).length > 1 &&
                !focusedPlayer
            ) {
                const partyBuffsObj = getPartyBuffs(
                    players,
                    encounterDamageStats.topDamageDealt,
                    encounterPartyInfo,
                    tempGroupedSynergies
                );

                if (localPlayer && $settings.meter.pinSelfParty) {
                    localPlayerInP1 = encounterPartyInfo[0].some((player) => player === localPlayer);
                }

                if (Object.keys(encounterPartyInfo).length > 2) {
                    localPlayerInP1 = true;
                }

                parties = partyBuffsObj.parties;
                partyGroupedSynergies = [...partyBuffsObj.partyGroupedSynergies];
                partyPercentages = partyBuffsObj.partyPercentages;
                partyBuffs = partyBuffsObj.partyBuffs;
            }
        }

        groupedSynergies = tempGroupedSynergies;
    });
</script>

{#if $settings.meter.splitPartyBuffs && parties.length > 1 && partyGroupedSynergies.length > 1 && parties.length === partyGroupedSynergies.length && tab === MeterTab.PARTY_BUFFS && !focusedPlayer}
    <div class="flex flex-col" class:flex-col-reverse={!localPlayerInP1} id="live-meter-table">
        {#each partyGroupedSynergies as [partyId, synergies], i (partyId)}
            {#if parties[i] && parties[i].length > 0}
                <table class="w-full table-fixed">
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
                                animate:flip={{ duration: 200 }}
                                onclick={() => inspectPlayer(player.name)}>
                                <PartyBuffRow {player} {playerBuffs} percentage={partyPercentages[i][playerIndex]} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        {/each}
    </div>
{:else}
    <table class="relative w-full table-fixed" id="live-meter-table">
        <thead class="sticky top-0 z-40 h-6">
            <tr class="bg-zinc-900">
                <th class="w-7 px-2 font-normal"></th>
                <th class="w-14 px-2 text-left font-normal"></th>
                <th class="w-full"></th>
                {#each [...groupedSynergies] as [id, synergies] (id)}
                    <BuffHeader {synergies} />
                {:else}
                    <th class="font-normal w-20">No Buffs</th>
                {/each}
            </tr>
        </thead>
        <tbody oncontextmenu={handleRightClick}>
            {#if !focusedPlayer}
                {#each players as player, i (player.name)}
                    <tr
                        class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                        animate:flip={{ duration: 200 }}
                        onclick={() => inspectPlayer(player.name)}>
                        <BuffRow {player} {groupedSynergies} percentage={percentages[i]} />
                    </tr>
                {/each}
            {:else}
                <BuffSkillBreakdown {groupedSynergies} player={focusedPlayer} {tab} />
            {/if}
        </tbody>
    </table>
{/if}
