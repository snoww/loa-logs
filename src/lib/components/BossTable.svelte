<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { flip } from "svelte/animate";
  import QuickTooltip from "./QuickTooltip.svelte";
  import BossRow from "./BossRow.svelte";

  interface Props {
    enc: EncounterState;
    inspectBoss: (boss: string) => void;
  }

  let { enc, inspectBoss }: Props = $props();

  let mostDamageDealtBoss = $derived(enc.bosses[0]?.damageStats.damageDealt ?? 1);
  let bossDamageDealtPercentages = $derived(
    enc.bosses.map((boss) => (boss.damageStats.damageDealt / mostDamageDealtBoss) * 100)
  );
</script>

<table class="relative isolate w-full table-fixed">
  <thead class="sticky top-0 z-40 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
    <tr class="bg-neutral-900 tracking-tight">
      <th class="w-14 px-2 text-left font-normal"></th>
      <th class="w-full"></th>
      <th class="w-14 font-normal">
        <QuickTooltip tooltip="Damage Dealt">DMG</QuickTooltip>
      </th>
      <th class="w-14 font-normal">
        <QuickTooltip tooltip="Damage per second">DPS</QuickTooltip>
      </th>
    </tr>
  </thead>
  <tbody class="relative z-10 text-neutral-200">
    {#each enc.bosses as boss, i (boss.name)}
      <tr
        class="h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
        animate:flip={{ duration: 200 }}
        onclick={() => inspectBoss(boss.name)}
      >
        <BossRow {boss} {enc} width={bossDamageDealtPercentages[i]} index={i} />
      </tr>
    {/each}
  </tbody>
</table>
