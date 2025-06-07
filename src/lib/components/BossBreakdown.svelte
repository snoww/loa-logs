<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import type { Entity } from "$lib/types";
  import { flip } from "svelte/animate";
  import QuickTooltip from "./QuickTooltip.svelte";
  import Back from "./Back.svelte";
  import BossBreakdownRow from "./BossBreakdownRow.svelte";

  interface Props {
    enc: EncounterState;
    boss: Entity;
    handleRightClick: () => void;
  }

  let { enc, boss, handleRightClick }: Props = $props();
  let entityState = $derived(new EntityState(boss, enc));
</script>

<table class="relative isolate w-full table-fixed">
  <thead class="sticky top-0 z-40 h-6 {enc.live ? 'sticky top-0 backdrop-blur-lg' : ''}">
    <tr class="bg-neutral-900 tracking-tighter">
      <th class="w-7 px-2 font-normal">
        <Back {handleRightClick} />
      </th>
      <th class="w-14 px-2 text-left font-normal"></th>
      <th class="w-full"></th>
      <th class="w-12 font-normal">
        <QuickTooltip tooltip="Damage Dealt">DMG</QuickTooltip>
      </th>
      <th class="w-12 font-normal">
        <QuickTooltip tooltip="Damage per second">DPS</QuickTooltip>
      </th>
      <th class="w-10 font-normal">
        <QuickTooltip tooltip="Damage Percentage">D%</QuickTooltip>
      </th>
      <th class="w-10 font-normal">
        <QuickTooltip tooltip="Total Casts">Casts</QuickTooltip>
      </th>
      <th class="w-10 font-normal">
        <QuickTooltip tooltip="Casts per minute">CPM</QuickTooltip>
      </th>
    </tr>
  </thead>
  <tbody oncontextmenu={handleRightClick} class="relative z-10 text-neutral-200">
    {#if boss}
      {#each entityState.skills as skill, i (skill.id)}
        <tr
          class="text-xxs h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}"
          animate:flip={{ duration: 200 }}
        >
          <BossBreakdownRow {entityState} {skill} width={entityState.skillDamagePercentages[i]} index={i} />
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
