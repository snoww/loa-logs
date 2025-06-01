<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "$lib/encounter.svelte";
  import { settings } from "$lib/stores.svelte";
  import DamageTakenRow from "./DamageTakenRow.svelte";

  interface Props {
    enc: EncounterState;
  }

  let { enc }: Props = $props();
</script>

<table class="relative isolate w-full table-fixed">
  <thead class="sticky top-0 z-40 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
    <tr class="bg-neutral-900 tracking-tight">
      <th class="w-7 px-2 font-normal"></th>
      <th class="w-14 px-2 text-left font-normal"></th>
      <th class="w-full"></th>
      <th class="w-28 font-normal">
        <QuickTooltip tooltip="Total Damage Taken">Damage Taken</QuickTooltip>
      </th>
    </tr>
  </thead>
  <tbody class="relative z-10">
    {#each enc.playerDamageTakenSorted as player, i (player.name)}
      <tr class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
        <DamageTakenRow {enc} {player} width={enc.playerDamageTakenPercentages[i]!} />
      </tr>
    {/each}
  </tbody>
</table>
