<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { BuffDetails, MeterTab, type Entity, type StatusEffect } from "$lib/types";
  import { getSynergyPercentageDetailsSum } from "$lib/utils/buffs";
  import QuickTooltip from "./QuickTooltip.svelte";
  import BuffDetailTooltip from "./tooltips/BuffDetailTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";
  import BuffSkillBreakdownRow from "./BuffSkillBreakdownRow.svelte";
  import { rgbLinearShadeAdjust } from "$lib/utils";

  interface Props {
    groupedSynergies: Map<string, Map<number, StatusEffect>>;
    player: Entity;
    enc: EncounterState;
    tab: MeterTab;
  }

  let { groupedSynergies, player, enc, tab }: Props = $props();
  let entityState = $derived(new EntityState(player, enc));

  let buffSummary: BuffDetails[] = $derived.by(() => {
    if (tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS) {
      return getSynergyPercentageDetailsSum(groupedSynergies, entityState);
    }
    return [];
  });
</script>

{#if tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS}
  <tr class="text-xxs h-7 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
    <td class="pl-1">
      <ClassTooltip entity={player} />
    </td>
    <td colspan="2">
      <div class="flex truncate">
        <QuickTooltip tooltip={entityState.name} class="truncate">
          {entityState.name}
        </QuickTooltip>
      </div>
    </td>
    {#if groupedSynergies.size > 0}
      {#each buffSummary as synergy (synergy.id)}
        <td class="px-1 text-center">
          {#if synergy.percentage}
            <BuffDetailTooltip buffDetails={synergy} />
          {/if}
        </td>
      {/each}
    {/if}
    <td
      class="absolute left-0 -z-10 h-7 w-full px-2 py-1"
      style="background-color: {settings.app.general.splitLines
        ? rgbLinearShadeAdjust(entityState.color, -0.2, 0.6)
        : `rgb(from ${entityState.color} r g b / 0.6)`}"
    ></td>
  </tr>
{/if}
{#each entityState.skills as skill, i (skill.id)}
  <BuffSkillBreakdownRow
    {groupedSynergies}
    {skill}
    {entityState}
    width={entityState.skillDamagePercentages[i]!}
    index={i}
  />
{/each}
