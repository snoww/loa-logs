<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { EntityType } from "$lib/types";
  import { flip } from "svelte/animate";
  import DamageMeterHeader from "./DamageMeterHeader.svelte";
  import PlayerRow from "./PlayerRow.svelte";

  interface Props {
    enc: EncounterState;
    inspectPlayer: (name: string) => void;
  }

  let { enc, inspectPlayer }: Props = $props();

  // array of party members
  const parties = $derived.by(() => {
    if (!settings.app.logs.splitPartyDamage || !enc.partyInfo || enc.live) {
      // all players (including esthers) in one party
      return [
        {
          title: "",
          members: enc.players.map((player, i) => ({
            entity: player,
            width: enc.playerDamagePercentages[i]
          }))
        }
      ];
    }

    // separate parties
    return enc.parties.map((party, partyId) => {
      return {
        title: `Party ${partyId + 1}`,
        members: party.map((player, i) => ({
          entity: player,
          width: enc.partyDamagePercentages[partyId]![i]
        }))
      };
    });
  });

  const esthers = $derived(enc.players.filter((entity) => entity.entityType === EntityType.ESTHER));
  const partiesWithEsthers = $derived.by(() => {
    // don't add esther party if esthers aren't shown or parties aren't split
    if (!esthers.length || !settings.app.general.showEsther || !settings.app.logs.splitPartyDamage || enc.live)
      return parties;

    return [
      ...parties,
      {
        title: "Esthers",
        members: esthers.map((esther) => ({
          entity: esther,
          width: (esther.damageStats.damageDealt / enc.topDamageDealt) * 100
        }))
      }
    ];
  });
</script>

<div class="flex flex-col space-y-2">
  {#each partiesWithEsthers.filter((p) => p.members.length > 0) as party}
    <table class="isolate w-full table-fixed">
      <thead class="z-40 h-6 {enc.live ? 'sticky top-0' : ''}">
        <tr class="bg-neutral-900">
          <th class="w-7 whitespace-nowrap px-2 font-normal tracking-tight">{party.title}</th>
          <DamageMeterHeader {enc} />
        </tr>
      </thead>
      <tbody class="relative z-10 text-neutral-200">
        {#each party.members as member (member.entity.name)}
          <tr
            animate:flip={{ duration: 200 }}
            class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
            onclick={() => inspectPlayer(member.entity.name)}
          >
            <PlayerRow {enc} entity={member.entity} width={member.width} />
          </tr>
        {/each}
      </tbody>
    </table>
  {/each}
</div>
