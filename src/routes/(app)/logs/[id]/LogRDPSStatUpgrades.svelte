<script lang="ts">
  import Card from "$lib/components/Card.svelte";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import Tooltipped from "$lib/components/Tooltipped.svelte";
  import { EFTable_PC } from "$lib/constants/EFTable_PC";
  import { IconInfo } from "$lib/icons";
  import type { ContributionSplit, Entity, StatDamageContribution } from "$lib/types";
  import { abbreviateNumber } from "$lib/utils";

  interface Props {
    player: Entity;
    split: ContributionSplit;
  }

  let { split, player }: Props = $props();

  const playerClassMainStat = $derived(["Strength", "Dexterity", "Intelligence"][EFTable_PC[player.classId]![0]]);

  const descs = $derived(
    (
      [
        ["criticalHitRate1percentDamage", "+1% Crit Rate"],
        ["criticalDamageRate1percentDamage", "+1% Crit Damage"],
        ["additionalDamage1percentDamage", "+1% Additional Damage"],
        ["evoDamage1percentDamage", "+1% Evolution-Type Damage"],
        ["weaponPower1000Damage", "+1,000 Weapon Power"],
        ["weaponPower1percentDamage", "+1% Weapon Power"],
        ["attackPower1000Damage", "+1,000 Attack Power"],
        ["attackPower1percentDamage", "+1% Attack Power"],
        ["mainStat1000Damage", `+1,000 ${playerClassMainStat}`]
      ] as [keyof ContributionSplit, string][]
    ).sort((a, b) => {
      const valA = split[a[0]] as StatDamageContribution | undefined;
      const valB = split[b[0]] as StatDamageContribution | undefined;
      const increaseA = valA ? valA.damageDoneByStatPlusValue - valA.damageDoneByStat : 0;
      const increaseB = valB ? valB.damageDoneByStatPlusValue - valB.damageDoneByStat : 0;
      return increaseB - increaseA;
    })
  );
</script>

{#snippet entry(key: keyof ContributionSplit, name: string)}
  {@const val = split[key] as StatDamageContribution | undefined}
  {#if val}
    {@const damageIncrease = Math.max(val.damageDoneByStatPlusValue - val.damageDoneByStat, 0)}
    {@const increasePct = damageIncrease / val.damageDoneByStat}
    <span class="text-sm">{name}</span>
    {#if damageIncrease !== 0}
      <span class="text-right font-mono text-sm">
        +{abbreviateNumber(damageIncrease)} ({(increasePct * 100).toFixed(2)}%)
      </span>
    {:else}
      <QuickTooltip
        tooltip="Stat increase values are not collected for logs recorded in low performance mode."
        class="cursor-default text-right text-sm text-neutral-500 underline decoration-dashed"
      >
        N/A
      </QuickTooltip>
    {/if}
  {/if}
{/snippet}

<Card class="mt-4">
  <div class="flex items-center justify-between bg-black/10 px-3 py-2 font-medium">
    <div>Damage Increase Potential</div>
    <Tooltipped>
      {#snippet tooltip()}
        <div class="flex max-w-[400px] flex-col gap-2 text-left">
          <span class="text-sm">
            Estimates on how much additional damage you would deal if you gained additional stats. These values are
            calculated by replaying your combat with simulated stat increases and comparing the damage done in those
            simulations to your actual damage.
          </span>
          <span class="text-sm">
            These estimates are rough values and are primarily meant to give a general overview of the impact of each
            stat on your damage. They are only based on this specific log, so their values may vary for logs with
            different RNG or party compositions.
          </span>
          <span class="text-sm">
            Due to the way that Lost Ark combat works, the damage increase gained from e.g. +10% crit rate is not the
            same as 10 times the damage increase from +1% crit rate, so these values should not be directly
            extrapolated.
          </span>
        </div>
      {/snippet}
      <IconInfo class="size-4" />
    </Tooltipped>
  </div>

  <div class="grid grid-cols-[1fr_max-content] gap-1 p-2">
    <span class="text-sm text-neutral-400">Stat Increase</span>
    <span class="text-right text-sm text-neutral-400">Expected Gain</span>

    {#each descs as desc}
      {@render entry(desc[0], desc[1])}
    {/each}
  </div>
</Card>
