<script lang="ts">
    import { calculatePartyWidth, filterStatusEffects, getPartyShields } from "$lib/utils/buffs";
    import { type EncounterDamageStats, type Entity, ShieldDetails, ShieldTab, type StatusEffect } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import ShieldHeader from "$lib/components/shared/ShieldHeader.svelte";
    import LogPartyShieldRow from "$lib/components/logs/LogPartyShieldRow.svelte";
    import { tooltip } from "$lib/utils/tooltip";

    export let players: Array<Entity>;
    export let encounterDamageStats: EncounterDamageStats;

    let tab = ShieldTab.GIVEN;

    let groupedSheilds = new Map<string, Map<number, StatusEffect>>();

    let parties = new Array<Array<Entity>>();
    let partyGroupedShields = new Map<string, Set<string>>();
    let partyPercentages = new Array<number[]>();
    let partyShields = new Map<string, Map<string, Array<ShieldDetails>>>();

    let vw: number;
    let partyWidths: { [key: string]: string };

    $: {
        for (const [id, buff] of Object.entries(encounterDamageStats.appliedShieldBuffs)) {
            filterStatusEffects(groupedSheilds, buff, Number(id), null, null, false, true);
        }
        groupedSheilds = new Map([...groupedSheilds.entries()].sort());
        if (encounterDamageStats.misc?.partyInfo) {
            let obj = getPartyShields(players, encounterDamageStats.misc.partyInfo, groupedSheilds, tab);
            parties = obj.parties;
            partyGroupedShields = obj.partyGroupedShields;
            partyPercentages = obj.partyPercentages;
            partyShields = obj.partyShields;
        }
        if (partyGroupedShields.size > 0) {
            const remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
            partyWidths = calculatePartyWidth(partyGroupedShields, remToPx, vw);
        }
    }
</script>

<svelte:window bind:innerWidth={vw} />
<div class="flex items-center divide-x divide-gray-600">
    <button
        class="rounded-sm border-t border-t-gray-600 px-2 py-1"
        class:bg-accent-900={tab === ShieldTab.GIVEN}
        class:bg-gray-700={tab !== ShieldTab.GIVEN}
        on:click={() => {
            tab = ShieldTab.GIVEN;
        }}
        use:tooltip={{ content: "Total amount of shields given by each skill" }}>
        Given
    </button>
    <button
        class="rounded-sm border-t border-t-gray-600 px-2 py-1"
        class:bg-accent-900={tab === ShieldTab.RECEIVED}
        class:bg-gray-700={tab !== ShieldTab.RECEIVED}
        on:click={() => {
            tab = ShieldTab.RECEIVED;
        }}
        use:tooltip={{ content: "Total amount of shields received from each skill" }}>
        Received
    </button>
    <button
        class="rounded-sm border-t border-t-gray-600 px-2 py-1"
        class:bg-accent-900={tab === ShieldTab.E_GIVEN}
        class:bg-gray-700={tab !== ShieldTab.E_GIVEN}
        on:click={() => {
            tab = ShieldTab.E_GIVEN;
        }}
        use:tooltip={{ content: "Total damage blocked of each shield" }}>
        Total Blocked
    </button>
    <button
        class="rounded-sm border-t border-t-gray-600 px-2 py-1"
        class:bg-accent-900={tab === ShieldTab.E_RECEIVED}
        class:bg-gray-700={tab !== ShieldTab.E_RECEIVED}
        on:click={() => {
            tab = ShieldTab.E_RECEIVED;
        }}
        use:tooltip={{ content: "Damage blocked by each shield" }}>
        Blocked Breakdown
    </button>
</div>
<div class="flex flex-col space-y-2">
    {#each [...partyGroupedShields] as [partyId, synergies], i (partyId)}
        {#if parties[i] && parties[i].length > 0}
            <table class="table-fixed" style="width: {partyWidths[partyId]};">
                <thead class="z-40 h-6" id="buff-head">
                    <tr class="bg-zinc-900">
                        {#if parties.length > 1}
                            <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Party {+partyId + 1}</th>
                        {:else}
                            <th class="w-7 px-2 font-normal" />
                        {/if}
                        <th class="w-20 px-2 text-left font-normal" />
                        <th class="w-full" />
                        <th class="w-20 font-normal">Total</th>
                        {#each synergies as synergy (synergy)}
                            {@const syns = groupedSheilds.get(synergy) || new Map()}
                            <ShieldHeader shields={syns} />
                        {/each}
                    </tr>
                </thead>
                <tbody class="relative z-10">
                    {#each parties[i] as player, playerIndex (player.name)}
                        {@const shields = partyShields.get(partyId)?.get(player.name) ?? []}
                        <tr class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
                            <LogPartyShieldRow
                                {player}
                                playerShields={shields}
                                percentage={partyPercentages[i][playerIndex]} />
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    {/each}
</div>
