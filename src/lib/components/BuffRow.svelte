<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
  import { addBardBubbles, supportSkills } from "$lib/utils/buffs";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import QuickTooltip from "./QuickTooltip.svelte";
  import BuffDetailTooltip from "./tooltips/BuffDetailTooltip.svelte";
  import ClassTooltip from "./tooltips/ClassTooltip.svelte";
  import { customRound } from "$lib/utils";

  interface Props {
    enc: EncounterState;
    player: Entity;
    groupedSynergies: Map<string, Map<number, StatusEffect>>;
    percentage: number;
  }

  let { enc, player, groupedSynergies, percentage }: Props = $props();

  let entityState = $derived(new EntityState(player, enc));

  let synergyPercentageDetails: Array<BuffDetails> = $derived.by(() => {
    let damageDealt = player.damageStats.damageDealt;
    let damageDealtWithoutHA = player.damageStats.damageDealt - (player.damageStats.hyperAwakeningDamage ?? 0);
    if (groupedSynergies.size > 0) {
      let tempSynergyPercentageDetails: Array<BuffDetails> = [];
      groupedSynergies.forEach((synergies, key) => {
        let synergyDamage = 0;
        let buff = new BuffDetails();
        let isHat = false;
        synergies.forEach((syn, id) => {
          if (supportSkills.haTechnique.includes(id)) {
            isHat = true;
          }
          if (player.damageStats.buffedBy[id]) {
            let b = new Buff(
              syn.source.icon,
              customRound((player.damageStats.buffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) * 100),
              syn.source.skill?.icon
            );
            addBardBubbles(key, b, syn);
            buff.buffs.push(b);
            synergyDamage += player.damageStats.buffedBy[id];
          } else if (player.damageStats.debuffedBy[id]) {
            buff.buffs.push(
              new Buff(
                syn.source.icon,
                customRound((player.damageStats.debuffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) * 100),
                syn.source.skill?.icon
              )
            );
            synergyDamage += player.damageStats.debuffedBy[id];
          }
        });

        if (synergyDamage > 0) {
          buff.percentage = customRound((synergyDamage / (isHat ? damageDealt : damageDealtWithoutHA)) * 100);
        }
        tempSynergyPercentageDetails.push(buff);
      });

      return tempSynergyPercentageDetails;
    }
    return [];
  });

  const tweenedValue = new Tween(enc.live ? 0 : percentage, {
    duration: 400,
    easing: cubicOut
  });

  $effect(() => {
    tweenedValue.set(percentage ?? 0);
  });

  let alpha = $derived(enc.live && !settings.app.meter.showClassColors ? 0 : 0.6);
</script>

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
  {#each synergyPercentageDetails as synergy}
    <td class="text-sm px-1 text-center text-neutral-200">
      {#if synergy.percentage}
        <BuffDetailTooltip buffDetails={synergy} />
      {/if}
    </td>
  {/each}
{/if}
<td
  class="absolute left-0 -z-10 h-7 px-2 py-1"
  style="background-color: rgb(from {entityState.color} r g b / {alpha}); width: {tweenedValue.current}%"
></td>
