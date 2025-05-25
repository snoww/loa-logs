<script lang="ts">
  import { BuffState } from "$lib/buffs.svelte";
  import LogPartyShieldRow from "$lib/components/logs/LogPartyShieldRow.svelte";
  import ShieldHeader from "$lib/components/shared/ShieldHeader.svelte";
  import type { EncounterState } from "$lib/encounter.svelte";
  import { ShieldTab } from "$lib/types";
  import { calculatePartyWidth } from "$lib/utils/buffs";
  import { settings } from "$lib/utils/settings";
  import { tooltip } from "$lib/utils/tooltip";

  interface Props {
    enc: EncounterState;
  }

  let { enc }: Props = $props();
  let buffs = $derived(new BuffState(enc));

  let tab = $state(ShieldTab.GIVEN);

  $effect.pre(() => {
    buffs.setShieldTab(tab);
  });

  let vw: number = $state(0);
  let partyWidths: { [key: string]: string } = $derived.by(() => {
    if (buffs.partyGroupedShields.size > 0) {
      const remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
      return calculatePartyWidth(buffs.partyGroupedShields, remToPx, vw);
    }
    return {};
  });
</script>

<svelte:window bind:innerWidth={vw} />
<div class="flex items-center divide-x divide-gray-600">
  <button
    class="rounded-xs border-t border-t-gray-600 px-2 py-1"
    class:bg-accent-900={tab === ShieldTab.GIVEN}
    class:bg-gray-700={tab !== ShieldTab.GIVEN}
    onclick={() => {
      tab = ShieldTab.GIVEN;
    }}
    use:tooltip={{ content: "Total amount of shields given by each skill" }}
  >
    Given
  </button>
  <button
    class="rounded-xs border-t border-t-gray-600 px-2 py-1"
    class:bg-accent-900={tab === ShieldTab.RECEIVED}
    class:bg-gray-700={tab !== ShieldTab.RECEIVED}
    onclick={() => {
      tab = ShieldTab.RECEIVED;
    }}
    use:tooltip={{ content: "Total amount of shields received from each skill" }}
  >
    Received
  </button>
  <button
    class="rounded-xs border-t border-t-gray-600 px-2 py-1"
    class:bg-accent-900={tab === ShieldTab.E_GIVEN}
    class:bg-gray-700={tab !== ShieldTab.E_GIVEN}
    onclick={() => {
      tab = ShieldTab.E_GIVEN;
    }}
    use:tooltip={{ content: "Total damage blocked of each shield" }}
  >
    Total Blocked
  </button>
  <button
    class="rounded-xs border-t border-t-gray-600 px-2 py-1"
    class:bg-accent-900={tab === ShieldTab.E_RECEIVED}
    class:bg-gray-700={tab !== ShieldTab.E_RECEIVED}
    onclick={() => {
      tab = ShieldTab.E_RECEIVED;
    }}
    use:tooltip={{ content: "Damage blocked by each shield" }}
  >
    Blocked Breakdown
  </button>
</div>
<div class="flex flex-col space-y-2">
  {#if enc.partyInfo && buffs.partyGroupedShields.size > 0 && buffs.shieldParties.length > 0}
    {#each buffs.partyGroupedShields as [partyId, synergies], i (partyId)}
      {#if buffs.shieldParties[i] && buffs.shieldParties[i].length > 0}
        <table class="table-fixed" style="width: {partyWidths[partyId]};">
          <thead class="z-40 h-6" id="buff-head">
            <tr class="bg-zinc-900">
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
              <tr class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
                <LogPartyShieldRow
                  {enc}
                  {player}
                  playerShields={shields}
                  percentage={buffs.shieldPartyPercentages[i][playerIndex]}
                />
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/each}
  {/if}
</div>
