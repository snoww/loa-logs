<script lang="ts">
  import { BuffState } from "$lib/buffs.svelte.js";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { ShieldTab } from "$lib/types";
  import ShieldHeader from "$lib/components/ShieldHeader.svelte";
  import LogPartyShieldRow from "./LogPartyShieldRow.svelte";

  interface Props {
    enc: EncounterState;
  }

  let { enc }: Props = $props();
  let buffs = $derived(new BuffState(enc));

  let tab = $state(ShieldTab.GIVEN);

  $effect.pre(() => {
    buffs.setShieldTab(tab);
  });
</script>

{#snippet shieldTab(selectedTab: ShieldTab, tabName: string, tooltip: string)}
  <QuickTooltip {tooltip}>
    <button
      class="text-nowrap rounded-lg px-2 py-1 transition {tab === selectedTab
        ? 'bg-accent-500/80'
        : 'hover:bg-neutral-800/40'}"
      onclick={() => {
        tab = selectedTab;
      }}
    >
      {tabName}
    </button>
  </QuickTooltip>
{/snippet}

<div class="mx-2 mb-2 flex w-fit items-center gap-1 truncate rounded-lg bg-neutral-700 md:mx-0">
  {@render shieldTab(ShieldTab.GIVEN, "Given", "Total amount of shields given by each skill")}
  {@render shieldTab(ShieldTab.RECEIVED, "Received", "Total amount of shields received from each skill")}
  {@render shieldTab(ShieldTab.E_GIVEN, "Total Blocked", "Total damage blocked of each shield")}
  {@render shieldTab(ShieldTab.E_RECEIVED, "Blocked Breakdown", "Damage blocked by each shield")}
</div>

<div class="flex flex-col space-y-2">
  {#if enc.partyInfo && buffs.partyGroupedShields.size > 0 && buffs.shieldParties.length > 0}
    {#each buffs.partyGroupedShields as [partyId, synergies], i (partyId)}
      {#if buffs.shieldParties[i] && buffs.shieldParties[i].length > 0}
        <table class="isolate w-full table-fixed">
          <thead class="z-40 h-6" id="buff-head">
            <tr class="bg-neutral-900">
              {#if buffs.shieldParties.length > 1}
                <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Party {+partyId + 1}</th>
              {:else}
                <th class="w-7 px-2 font-normal"></th>
              {/if}
              <th class="w-20 px-2 text-left font-normal"></th>
              <th class="w-full"></th>
              <th class="w-20 font-normal">Total</th>
              {#each synergies as synergy (synergy)}
                {@const syns = buffs.groupedShields.get(synergy) || new Map()}
                <ShieldHeader shields={syns} />
              {/each}
            </tr>
          </thead>
          <tbody class="relative z-10">
            {#each buffs.shieldParties[i] as player, playerIndex (player.name)}
              {@const shields = buffs.partyShields.get(partyId)?.get(player.name) ?? []}
              <tr class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
                <LogPartyShieldRow
                  {enc}
                  {player}
                  playerShields={shields}
                  percentage={buffs.shieldPartyPercentages[i]![playerIndex]!}
                />
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/each}
  {/if}
</div>
