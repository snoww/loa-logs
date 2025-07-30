<script lang="ts">
  import { BuffState } from "$lib/buffs.svelte.js";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { EntityType, MeterTab, type Entity } from "$lib/types";
  import { flip } from "svelte/animate";
  import Back from "./Back.svelte";
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
</script>

{#if enc.curSettings.splitPartyBuffs && enc.parties.length > 1 && buffs.partyGroupedSynergies.size > 1 && enc.parties.length === buffs.partyGroupedSynergies.size && tab === MeterTab.PARTY_BUFFS && !focusedPlayer}
  <div class="flex flex-col {enc.live ? '' : 'gap-2'}">
    {#each buffs.partyGroupedSynergies as [partyId, synergies], i (partyId)}
      {#if enc.parties[i] && enc.parties[i].length > 0}
        <table
          class="isolate w-full table-fixed {enc.live &&
          settings.app.meter.pinSelfParty &&
          buffs.enc.parties[i].some((player) => player.name === enc.localPlayer)
            ? 'order-first'
            : ''}"
        >
          <thead class="z-40 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
            <tr class="bg-neutral-900">
              <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">Party {+partyId + 1}</th>
              <th class="w-20 px-2 text-left font-normal"></th>
              <th class="w-full"></th>
              {#each [...synergies] as synergy (synergy)}
                {@const syns = buffs.groupedSynergies.get(synergy) || new Map()}
                <BuffHeader buffs={syns} />
              {/each}
            </tr>
          </thead>
          <tbody class="relative z-10">
            {#each enc.parties[i] as player, playerIndex (player.name)}
              {@const playerBuffs = buffs.partyBuffs.get(partyId)?.get(player.name) ?? []}
              <tr
                animate:flip={{ duration: 200 }}
                class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
                onclick={() => inspectPlayer(player.name)}
              >
                <PartyBuffRow {player} {enc} {playerBuffs} percentage={enc.partyDamagePercentages[i][playerIndex]!} />
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/each}
  </div>
{:else}
  <table class="isolate w-full table-fixed">
    <thead class="relative z-40 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
      <tr class="bg-neutral-900">
        <th class="w-7 px-2 font-normal">
          {#if focusedPlayer}
            <Back {handleRightClick} />
          {/if}
        </th>
        <th class="w-20 px-2 text-left font-normal"></th>
        <th class="w-full"></th>
        {#each buffs.groupedSynergies as [id, synergies] (id)}
          <BuffHeader buffs={synergies} />
        {:else}
          <th class="font-normal w-20">No Buffs</th>
        {/each}
      </tr>
    </thead>
    <tbody oncontextmenu={handleRightClick} class="relative z-10">
      {#if !focusedPlayer}
        {#each buffs.players as player, i (player.name)}
          <tr
            animate:flip={{ duration: 200 }}
            class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
            onclick={() => inspectPlayer(player.name)}
          >
            <BuffRow {enc} {player} groupedSynergies={buffs.groupedSynergies} percentage={buffs.percentages[i]!} />
          </tr>
        {/each}
      {:else}
        <BuffSkillBreakdown {enc} groupedSynergies={buffs.groupedSynergies} player={focusedPlayer} {tab} />
      {/if}
    </tbody>
  </table>
{/if}
