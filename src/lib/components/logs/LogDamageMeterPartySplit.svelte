<script lang="ts">
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityType, type Entity } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import DamageMeterHeader from "../shared/DamageMeterHeader.svelte";
    import PlayerRow from "../shared/PlayerRow.svelte";

    interface Props {
        enc: EncounterState;
        inspectPlayer: (name: string) => void;
    }

    let { enc, inspectPlayer }: Props = $props();

    let parties = $state(new Array<Array<Entity>>());
    let partyPercentages = $state(new Array<number[]>());
    let anyPartyDead = $state(new Array<boolean>());
    let multipleDeaths = $state(false);

    let esthers = $derived(enc.players.filter((entity) => entity.entityType === EntityType.ESTHER));

    $effect(() => {
        if (enc.partyInfo) {
            let partyData = new Array<Array<Entity>>();
            const partyInfo = Object.entries(enc.partyInfo);
            if (partyInfo.length >= 2) {
                for (const [partyIdStr, names] of partyInfo) {
                    const partyId = Number(partyIdStr);
                    partyData[partyId] = [];
                    anyPartyDead[partyId] = false;
                    for (const name of names) {
                        const player = enc.players.find((player) => player.name === name);
                        if (player) {
                            partyData[partyId].push(player);
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
                    if (partyData[partyId] && partyData[partyId].length > 0) {
                        partyData[partyId].sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                        partyPercentages[partyId] = partyData[partyId].map(
                            (player) => (player.damageStats.damageDealt / enc.topDamageDealt) * 100
                        );
                    }
                }
            } else {
                partyData[0] = enc.players;
            }

            parties = partyData;
        }
    });
</script>

<div class="flex flex-col space-y-2">
    {#each parties as party, partyId}
        {#if party && party.length > 0}
            <table class="w-full table-fixed">
                <thead class="z-40 h-6">
                    <tr class="bg-zinc-900">
                        <th class="w-7 px-2 font-normal tracking-tight whitespace-nowrap">Party {+partyId + 1}</th>
                        <DamageMeterHeader anyDead={anyPartyDead[partyId]} {enc} />
                    </tr>
                </thead>
                <tbody class="relative z-10">
                    {#each party as player, playerIndex (player.name)}
                        <tr
                            class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                            onclick={() => inspectPlayer(player.name)}>
                            <PlayerRow
                                {enc}
                                entity={player}
                                width={partyPercentages[partyId][playerIndex]}
                                anyDead={anyPartyDead[partyId]} />
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
                    <th class="w-7 px-2 font-normal tracking-tight whitespace-nowrap">Esthers</th>
                    <th class="w-20 px-2 text-left font-normal"></th>
                    <th class="w-full"></th>
                    <DamageMeterHeader anyDead={false} {enc} />
                </tr>
            </thead>
            <tbody class="relative z-10">
                {#each esthers as esther (esther.name)}
                    <tr
                        class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                        onclick={() => inspectPlayer(esther.name)}>
                        <PlayerRow
                            {enc}
                            entity={esther}
                            width={(esther.damageStats.damageDealt / enc.topDamageDealt) * 100}
                            anyDead={false} />
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
