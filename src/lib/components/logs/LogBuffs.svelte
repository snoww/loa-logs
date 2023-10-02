<script lang="ts">
    import {
        type EncounterDamageStats,
        type Entity,
        MeterTab,
        type StatusEffect,
        EntityType,
        BuffDetails,
        Buff
    } from "$lib/types";
    import { filterStatusEffects } from "$lib/utils/buffs";
    import LogBuffHeader from "./LogBuffHeader.svelte";
    import LogPartyBuffRow from "./LogPartyBuffRow.svelte";
    import LogBuffBreakdown from "./LogBuffBreakdown.svelte";
    import { settings } from "$lib/utils/settings";
    import LogPartyBuffHeader from "./LogPartyBuffHeader.svelte";
    import LogBuffRow from "./LogBuffRow.svelte";
    import { round } from "$lib/utils/numbers";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats;
    export let players: Array<Entity>;
    export let focusedPlayer: Entity | null = null;
    export let inspectPlayer: (name: string) => void;

    if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
        focusedPlayer = null;
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
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer, tab, $settings.buffs.default);
        }
    }
    for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
        if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
            continue;
        }
        if (debuff.category === "debuff") {
            filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer, tab, $settings.buffs.default);
        }
    }
    groupedSynergies = new Map([...groupedSynergies.entries()].sort());

    let parties = new Array<Array<Entity>>();
    let partyGroupedSynergies = new Map<string, Set<string>>();
    let partyPercentages = new Array<number[]>();

    let partyBuffs = new Map<string, Map<string, Array<BuffDetails>>>();

    let vw: number;
    let partyWidths: { [key: string]: string };

    if ($settings.logs.splitPartyBuffs && encounterDamageStats.misc?.partyInfo) {
        let partyInfo = Object.entries(encounterDamageStats.misc.partyInfo);
        if (partyInfo.length >= 2) {
            for (const [partyIdStr, names] of partyInfo) {
                const partyId = Number(partyIdStr);
                for (const name of names) {
                    const player = players.find((player) => player.name === name);
                    if (player) {
                        parties[partyId] = parties[partyId] || [];
                        parties[partyId].push(player);
                    }
                }
                if (parties[partyId]) {
                    parties[partyId].sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                    partyPercentages[partyId] = parties[partyId].map(
                        (player) => (player.damageStats.damageDealt / encounterDamageStats.topDamageDealt) * 100
                    );
                }
            }
        } else {
            parties[0] = players;
        }

        if (!focusedPlayer && groupedSynergies.size > 0 && parties.length > 1) {
            parties.forEach((party, partyId) => {
                partyGroupedSynergies.set(partyId.toString(), new Set<string>());
                let partySyns = partyGroupedSynergies.get(partyId.toString())!;
                for (const player of party) {
                    groupedSynergies.forEach((synergies, key) => {
                        synergies.forEach((_, id) => {
                            if (player.damageStats.buffedBy[id] || player.damageStats.debuffedBy[id]) {
                                partySyns.add(key);
                            }
                        });
                    });
                }
            });

            parties.forEach((party, partyId) => {
                partyBuffs.set(partyId.toString(), new Map<string, Array<BuffDetails>>());
                for (const player of party) {
                    partyBuffs.get(partyId.toString())!.set(player.name, []);
                    let playerBuffs = partyBuffs.get(partyId.toString())!.get(player.name)!;
                    partyGroupedSynergies.get(partyId.toString())?.forEach((key) => {
                        let buffDetails = new BuffDetails();
                        buffDetails.id = key;
                        let buffDamage = 0;
                        let buffs = groupedSynergies.get(key) || new Map();
                        buffs.forEach((syn, id) => {
                            if (player.damageStats.buffedBy[id]) {
                                buffDetails.buffs.push(
                                    new Buff(
                                        syn.source.icon,
                                        round((player.damageStats.buffedBy[id] / player.damageStats.damageDealt) * 100),
                                        syn.source.skill?.icon
                                    )
                                );
                                buffDamage += player.damageStats.buffedBy[id];
                            } else if (player.damageStats.debuffedBy[id]) {
                                buffDetails.buffs.push(
                                    new Buff(
                                        syn.source.icon,
                                        round(
                                            (player.damageStats.debuffedBy[id] / player.damageStats.damageDealt) * 100
                                        ),
                                        syn.source.skill?.icon
                                    )
                                );
                                buffDamage += player.damageStats.debuffedBy[id];
                            }
                        });
                        if (buffDamage > 0) {
                            buffDetails.percentage = round((buffDamage / player.damageStats.damageDealt) * 100);
                        }
                        playerBuffs.push(buffDetails);
                    });
                }
            });
        }
    }
    console.log(parties)

    $: {
        partyWidths = {};
        let remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
        partyGroupedSynergies.forEach((synergies, partyId) => {
            const widthRem = synergies.size * 3.5 + 10;
            const widthPx = widthRem * remToPx;
            if (widthPx > vw - 2 * remToPx) {
                partyWidths[partyId] = `${widthRem}rem`;
            } else {
                partyWidths[partyId] = "calc(100vw - 4.5rem)";
            }
        });
    }
</script>

<svelte:window bind:innerWidth={vw} />
{#if $settings.logs.splitPartyBuffs && parties.length > 1 && tab === MeterTab.PARTY_BUFFS && !focusedPlayer}
    <div class="flex flex-col space-y-2">
        {#each [...partyGroupedSynergies] as [partyId, synergies], i (partyId)}
            <table class="table-fixed" style="width: {partyWidths[partyId]};">
                <thead class="z-40 h-6" id="buff-head">
                    <tr class="bg-zinc-900">
                        <th class="w-7 whitespace-nowrap px-2 font-normal">Party {+partyId + 1}</th>
                        <th class="w-20 px-2 text-left font-normal" />
                        <th class="w-full" />
                        {#each [...synergies] as synergy (synergy)}
                            {@const syns = groupedSynergies.get(synergy) || new Map()}
                            <LogPartyBuffHeader synergies={syns} />
                        {/each}
                    </tr>
                </thead>
                <tbody class="relative z-10">
                    {#each parties[i] as player, playerIndex (player.name)}
                        {@const playerBuffs = partyBuffs.get(partyId)?.get(player.name) ?? []}
                        <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
                            <LogPartyBuffRow {player} {playerBuffs} percentage={partyPercentages[i][playerIndex]} />
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/each}
    </div>
{:else}
    <table class="w-full table-fixed">
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
        <tbody class="relative z-10">
            {#if !focusedPlayer}
                {#each players as player, i (player.name)}
                    <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
                        <LogBuffRow {player} {groupedSynergies} percentage={percentages[i]} />
                    </tr>
                {/each}
            {:else}
                <LogBuffBreakdown {groupedSynergies} player={focusedPlayer} {tab} />
            {/if}
        </tbody>
    </table>
{/if}
