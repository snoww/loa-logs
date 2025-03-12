<script lang="ts">
    import { BuffState } from "$lib/buffs.svelte";
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityType, MeterTab, type Entity } from "$lib/types";
    import { calculatePartyWidth } from "$lib/utils/buffs";
    import { settings } from "$lib/utils/settings";
    import { flip } from "svelte/animate";
    import BuffHeader from "./BuffHeader.svelte";
    import BuffRow from "./BuffRow.svelte";
    import BuffSkillBreakdown from "./BuffSkillBreakdown.svelte";
    import PartyBuffRow from "./PartyBuffRow.svelte";

    interface Props {
        tab: MeterTab;
        enc: EncounterState;
        focusedPlayer?: Entity;
        inspectPlayer: (name: string) => void;
        handleRightClick: () => void;
    }

    let { tab, enc, focusedPlayer = $bindable(), inspectPlayer, handleRightClick }: Props = $props();

    let buffs = $derived(new BuffState(enc));

    $effect(() => {
        if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
            focusedPlayer = undefined;
        } else {
            buffs.setFocusedPlayer(focusedPlayer);
        }
    });

    $effect(() => {
        buffs.setTab(tab);
    });

    let vw: number = $state(0);
    let partyWidths: Record<string, string> = $derived.by(() => {
        if (buffs.partyGroupedSynergies.size > 0) {
            const remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
            return calculatePartyWidth(buffs.partyGroupedSynergies, remToPx, vw);
        }
        return {};
    });
</script>

<svelte:window bind:innerWidth={vw} />
{#if enc.curSettings.splitPartyBuffs && buffs.buffParties.length > 1 && buffs.partyGroupedSynergies.size > 1 && buffs.buffParties.length === buffs.partyGroupedSynergies.size && tab === MeterTab.PARTY_BUFFS && !focusedPlayer}
    <div class="flex flex-col {enc.live ? '' : 'space-y-2'}" id="live-meter-table">
        {#each buffs.partyGroupedSynergies as [partyId, synergies], i (partyId)}
            {#if buffs.buffParties[i] && buffs.buffParties[i].length > 0}
                <table class="{enc.live ? 'w-full' : ''} table-fixed" style="width: {partyWidths[partyId]};">
                    <thead class="z-40 h-6" id="buff-head">
                        <tr class="bg-zinc-900">
                            <th class="w-7 px-2 font-normal tracking-tight whitespace-nowrap">Party {+partyId + 1}</th>
                            <th class="w-20 px-2 text-left font-normal"></th>
                            <th class="w-full"></th>
                            {#each [...synergies] as synergy (synergy)}
                                {@const syns = buffs.groupedSynergies.get(synergy) || new Map()}
                                <BuffHeader synergies={syns} />
                            {/each}
                        </tr>
                    </thead>
                    <tbody class="relative z-10">
                        {#each buffs.buffParties[i] as player, playerIndex (player.name)}
                            {@const playerBuffs = buffs.partyBuffs.get(partyId)?.get(player.name) ?? []}
                            <tr
                                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                                animate:flip={{ duration: 200 }}
                                onclick={() => inspectPlayer(player.name)}>
                                <PartyBuffRow
                                    {player}
                                    {enc}
                                    {playerBuffs}
                                    percentage={buffs.partyPercentages[i][playerIndex]} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        {/each}
    </div>
{:else}
    <table class="w-full table-fixed" id="live-meter-table">
        <thead class="{enc.live ? 'sticky top-0' : 'relative'} z-40 h-6">
            <tr class="bg-zinc-900">
                <th class="w-7 px-2 font-normal"></th>
                <th class="{enc.live ? 'w-14' : 'w-20'} px-2 text-left font-normal"></th>
                <th class="w-full"></th>
                {#each buffs.groupedSynergies as [id, synergies] (id)}
                    <BuffHeader {synergies} />
                {:else}
                    <th class="font-normal w-20">No Buffs</th>
                {/each}
            </tr>
        </thead>
        <tbody oncontextmenu={handleRightClick} class={enc.live ? "" : "relative z-10"}>
            {#if !focusedPlayer}
                {#each buffs.players as player, i (player.name)}
                    <tr
                        class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                        animate:flip={{ duration: 200 }}
                        onclick={() => inspectPlayer(player.name)}>
                        <BuffRow
                            {enc}
                            {player}
                            groupedSynergies={buffs.groupedSynergies}
                            percentage={buffs.percentages[i]} />
                    </tr>
                {/each}
            {:else}
                <BuffSkillBreakdown {enc} groupedSynergies={buffs.groupedSynergies} player={focusedPlayer} {tab} />
            {/if}
        </tbody>
    </table>
{/if}
