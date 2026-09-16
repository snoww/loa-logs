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
  const miscellaneousGains = $derived.by(() => {
    const gains: Record<"stagger" | "npc_damage" | "atropine", [string, StatDamageContribution][]> = {
      stagger: [],
      npc_damage: [],
      atropine: []
    };
    if (split.npcWindows && split.npcWindows.trackedHits > 0) {
      gains.stagger.push(
        ["Domination", split.npcWindows.domination],
        ["Broken Bone", split.npcWindows.brokenBone],
        ["Bracelet Effects", split.npcWindows.staggerCombatEffect]
      );
      gains.npc_damage.push(["Boss Damage Taken Increase", split.npcWindows.npcDamageTaken]);
    }
    if (split.atropineDamageBonus && split.atropineDamageBonus.damageDoneByStatPlusValue > 0) {
      gains.atropine.push(["Atropine Attack Power Bonus", split.atropineDamageBonus]);
    }
    return gains;
  });
  const hasMiscellaneousGains = $derived(Object.values(miscellaneousGains).some((arr) => arr.length > 0));
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

<Card>
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

{#snippet miscBlock(entries: [string, StatDamageContribution][], title: string)}
  {#if entries.length > 0}
    <span class="col-span-full text-sm text-neutral-400 not-first:mt-2">{title}</span>

    {#each entries as [name, value] (name)}
      {@const gain = value.damageDoneByStatPlusValue - value.damageDoneByStat}
      <span class="text-sm">{name}</span>
      <span class="text-right font-mono text-sm">
        +{abbreviateNumber(gain)} ({value.damageDoneByStat > 0
          ? ((100 * gain) / value.damageDoneByStat).toFixed(2)
          : "0.00"}%)
      </span>
    {/each}
  {/if}
{/snippet}

{#if hasMiscellaneousGains}
  <Card>
    <div class="flex items-center justify-between bg-black/10 px-3 py-2 font-medium">
      <div>Miscellaneous Damage Gains</div>
      <Tooltipped>
        {#snippet tooltip()}
          <div class="max-w-[400px] text-left text-sm">
            Damage gained compared with the same hits after removing only the listed bonus. These gains can overlap and
            should not be added together. Atropine measures attack power only, attack speed bonus portion is not
            included.
          </div>
        {/snippet}
        <IconInfo class="size-4" />
      </Tooltipped>
    </div>
    <div class="grid grid-cols-[1fr_max-content] gap-1 p-2">
      {@render miscBlock(miscellaneousGains.stagger, "Stagger Bonus Damage")}
      {@render miscBlock(miscellaneousGains.npc_damage, "Boss Mechanics")}
      {@render miscBlock(miscellaneousGains.atropine, "Battle Items")}
    </div>
  </Card>
{/if}
