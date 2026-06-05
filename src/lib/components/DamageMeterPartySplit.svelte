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
          sortable: true,
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
        sortable: true,
        members: party.map((player, i) => ({
          entity: player,
          width: enc.partyDamagePercentages[partyId]![i]
        }))
      };
    });
  });

  const esthers = $derived(enc.players.filter((entity) => entity.entityType === EntityType.ESTHER));
  const partiesWithEsthers = $derived.by(() => {
    let result = parties;
    // don't add esther party if esthers aren't shown or parties aren't split
    if (esthers.length && settings.app.general.showEsther && settings.app.logs.splitPartyDamage && !enc.live) {
      result = [
        ...result,
        {
          title: "Esthers",
          sortable: false,
          members: esthers.map((esther) => ({
            entity: esther,
            width: (esther.damageStats.damageDealt / enc.topDamageDealt) * 100
          }))
        }
      ];
    }
    // dark grenade synergy table — only in logs view, when rDPS column is enabled
    if (!enc.live && enc.curSettings.rdps && enc.darkGrenade) {
      const dg = enc.darkGrenade;
      const rdamage = dg.damageStats.rdpsDamageGiven;
      result = [
        ...result,
        {
          title: "Other",
          sortable: false,
          members: [
            {
              entity: dg,
              width: enc.topDamageDealt > 0 ? (rdamage / enc.topDamageDealt) * 100 : 0
            }
          ]
        }
      ];
    }
    return result;
  });
</script>

<div class="flex flex-col space-y-2">
  {#each partiesWithEsthers.filter((p) => p.members.length > 0) as party}
    <table class="isolate w-full table-fixed">
      <thead class="z-40 h-6 {enc.live ? 'sticky top-0' : ''}">
        <tr class="bg-neutral-900">
          <th class="w-7 px-2 font-normal tracking-tight whitespace-nowrap">{party.title}</th>
          <DamageMeterHeader {enc} sortable={party.sortable ?? true} />
        </tr>
      </thead>
      <tbody class="relative z-10 text-neutral-200">
        {#each party.members as member (member.entity.name)}
          {@const clickable = member.entity.entityType !== EntityType.DARK_GRENADE}
          <tr
            animate:flip={{ duration: 200 }}
            class="h-7 px-2 py-1 {clickable && settings.app.general.underlineHovered ? 'hover:underline' : ''}"
            onclick={clickable ? () => inspectPlayer(member.entity.name) : null}
          >
            <PlayerRow {enc} entity={member.entity} width={member.width} sortable={party.sortable} />
          </tr>
        {/each}
      </tbody>
    </table>
  {/each}
</div>
