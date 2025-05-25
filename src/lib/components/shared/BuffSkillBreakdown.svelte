<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { EntityState } from "$lib/entity.svelte";
  import { BuffDetails, MeterTab, type Entity, type StatusEffect } from "$lib/types";
  import { getSynergyPercentageDetailsSum } from "$lib/utils/buffs";
  import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
  import { classIconCache, settings } from "$lib/utils/settings";
  import { takingScreenshot } from "$lib/utils/stores";
  import { tooltip } from "$lib/utils/tooltip";
  import BuffSkillBreakdownRow from "./BuffSkillBreakdownRow.svelte";
  import BuffTooltipDetail from "./BuffTooltipDetail.svelte";

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
      return getSynergyPercentageDetailsSum(groupedSynergies, entityState.skills, player.damageStats);
    }
    return [];
  });
</script>

{#if tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS}
  <tr class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
    <td class="pl-1">
      <img
        class="table-cell size-5"
        src={$classIconCache[player.classId]}
        alt={player.class}
        use:tooltip={{ content: player.class }}
      />
    </td>
    <td colspan="2">
      <div class="truncate">
        <span use:tooltip={{ content: entityState.name }}>
          {entityState.name}
        </span>
      </div>
    </td>
    {#if groupedSynergies.size > 0}
      {#each buffSummary as synergy (synergy.id)}
        <td class="px-1 text-center">
          {#if synergy.percentage}
            <BuffTooltipDetail {synergy} />
          {/if}
        </td>
      {/each}
    {/if}
    <td
      class="absolute left-0 -z-10 h-7 w-full px-2 py-1"
      class:shadow-md={!$takingScreenshot}
      style="background-color: {$settings.general.splitLines
        ? RGBLinearShade(HexToRgba(entityState.color, 0.6))
        : HexToRgba(entityState.color, 0.6)}"
    ></td>
  </tr>
{/if}
{#each entityState.skills as skill, i (skill.id)}
  <BuffSkillBreakdownRow
    {groupedSynergies}
    {skill}
    {entityState}
    width={entityState.skillDamagePercentages[i]}
    index={i}
  />
{/each}
