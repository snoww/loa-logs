<script lang="ts">
  import Card from "$lib/components/Card.svelte";
  import Tooltipped from "$lib/components/Tooltipped.svelte";
  import { EFTable_ArkPassive } from "$lib/constants/EFTable_ArkPassive";
  import { Engrave } from "$lib/constants/Engrave";
  import { IconInfo } from "$lib/icons";
  import type { ContributionSplit } from "$lib/types";
  import { type Snippet } from "svelte";

  interface Props {
    split: ContributionSplit;
  }

  let { split }: Props = $props();
</script>

{#snippet efficiencySection(icon: string, name: string, tooltipSnippet: Snippet<[number]>, value: number)}
  <div class="efficiency-block flex flex-row items-center gap-2 border-neutral-950 p-2">
    <img
      alt=""
      class="size-10 rounded-sm"
      src={`https://cdn.ags.lol/icon/${icon}.png`}
      class:grayscale={value === -1}
    />
    <div class="flex flex-1 flex-col justify-center gap-2" class:opacity-50={value === -1}>
      <div class="flex flex-row justify-between">
        <div class="flex flex-row items-center gap-1 text-sm">
          {name}
          <Tooltipped>
            <IconInfo class="size-4" />
            {#snippet tooltip()}
              {@render tooltipSnippet(value)}
            {/snippet}
          </Tooltipped>
        </div>
        <div class="text-sm font-medium">{value === -1 ? "Not Equipped" : `${(value * 100).toFixed(1)}%`}</div>
      </div>
      <div class="w-full rounded-full bg-neutral-700">
        {#if value !== -1}
          <div class="h-2 rounded-full bg-accent-600" style={`width: ${(value * 100).toFixed(1)}%`}></div>
        {:else}
          <div class="h-2" style="width: 100%"></div>
        {/if}
      </div>
    </div>
  </div>
{/snippet}

{#snippet rcEfficiencyTooltip(val: number)}
  <div class="flex max-w-[400px] flex-col gap-2 text-left">
    <span class="text-sm">
      Represents how much of the total possible damage gained by the Raid Captain engraving was actually utilized. A
      value of 0% means that you were never above base movement speed, while a value of 100% means that you were at the
      movement speed cap for the entire fight. This value is calculated even if you were not running Raid Captain.
    </span>
    <span class="text-sm">
      Your total damage contribution from Raid Captain can be calculated by multiplying the utilization by the damage
      bonus given by the engraving. For your utilization of {(val * 100).toFixed(0)}%, expected damage gains are:
    </span>

    <ul class="list-none text-sm">
      <li>
        <span class="text-[#FE9600]">Legendary Raid Captain (0/20)</span>: {(val * (40 * 0.4)).toFixed(1)}% damage
        increase
      </li>
      <li>
        <span class="text-[#FE9600]">Legendary Raid Captain (5/20)</span>: {(val * (42 * 0.4)).toFixed(1)}% damage
        increase
      </li>
      <li>
        <span class="text-[#FE9600]">Legendary Raid Captain (10/20)</span>: {(val * (44 * 0.4)).toFixed(1)}% damage
        increase
      </li>
      <li>
        <span class="text-[#FE9600]">Legendary Raid Captain (15/20)</span>: {(val * (46 * 0.4)).toFixed(1)}% damage
        increase
      </li>
      <li>
        <span class="text-[#FA5D00]">Relic Raid Captain</span>: {(val * (48 * 0.4)).toFixed(1)}% damage increase
      </li>
    </ul>
  </div>
{/snippet}

{#snippet rcEfficiency()}
  {@const eff = split.raidCaptainEfficiency!.damageDoneByStat / split.raidCaptainEfficiency!.damageDoneByStatPlusValue}
  {@render efficiencySection(Engrave[254][1], "Raid Captain Efficiency", rcEfficiencyTooltip, eff)}
{/snippet}

{#snippet btEfficiencyTooltip(val: number)}
  <div class="flex max-w-[400px] flex-col gap-2 text-left">
    <span class="text-sm">
      Represents how much of the total possible damage gained by the Blunt Thorn Ark Passive node was actually utilized.
      A value of 0% means that you were never above 80% crit rate, while a value of 100% means that your uncapped crit
      rate always exceeded the maximum conversion threshold.
    </span>
    {#if val !== -1}
      <span class="text-sm">
        Your total damage contribution from Blunt Thorn can be calculated by multiplying the utilization by the damage
        conversion cap of the node. For your utilization of {(val * 100).toFixed(0)}%, the additional evolution-type
        damage given by Blunt Thorn is:
      </span>
      <div>
        <div class="text-sm">
          <b>Blunt Thorn Lv. 1</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(val * 52.5).toFixed(1)}% evolution-type damage</li>
          </ul>
        </div>
        <div class="text-sm">
          <b>Blunt Thorn Lv. 2</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(val * 75).toFixed(1)}% evolution-type damage</li>
          </ul>
        </div>
      </div>
    {:else}
      <span class="text-sm">This character does not have Blunt Thorn equipped, so no utilization was calculated.</span>
    {/if}
  </div>
{/snippet}

{#snippet btEfficiency()}
  {@const icon = EFTable_ArkPassive[1040100]![1]}
  {@const v = split.bluntThornEfficiency!}
  {#if !v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Blunt Thorn Utilization", btEfficiencyTooltip, -1)}
  {:else}
    {@const eff = v.damageDoneByStat / v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Blunt Thorn Utilization", btEfficiencyTooltip, eff)}
  {/if}
{/snippet}

{#snippet sbEfficiencyTooltip(val: number)}
  <div class="flex max-w-[400px] flex-col gap-2 text-left">
    <span class="text-sm">
      Represents how much of the total possible damage gained by the Supersonic Breakthrough Ark Passive node was
      actually utilized. A value of 0% means that you never gained any additional attack or movement speed, while a
      value of 100% means that your uncapped attack and movement speed always exceeded the maximum conversion threshold.
    </span>
    {#if val !== -1}
      <span class="text-sm">
        Your total damage contribution from Supersonic Breakthrough can be calculated by multiplying the utilization by
        the damage conversion cap of the node. For your utilization of {(val * 100).toFixed(0)}%, the additional
        evolution-type damage given by Supersonic Breakthrough is:
      </span>
      <div>
        <div class="text-sm">
          <b>Supersonic Breakthrough Lv. 1</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(val * 12).toFixed(1)}% evolution-type damage</li>
          </ul>
        </div>
        <div class="text-sm">
          <b>Supersonic Breakthrough Lv. 2</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(val * 24).toFixed(1)}% evolution-type damage</li>
          </ul>
        </div>
      </div>
    {:else}
      <span class="text-sm">
        This character does not have Supersonic Breakthrough equipped, so no utilization was calculated.
      </span>
    {/if}
  </div>
{/snippet}

{#snippet sbEfficiency()}
  {@const icon = EFTable_ArkPassive[1040200]![1]}
  {@const v = split.supersonicBreakthroughEfficiency!}
  {#if !v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Supersonic Breakthrough Utilization", sbEfficiencyTooltip, -1)}
  {:else}
    {@const eff = v.damageDoneByStat / v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Supersonic Breakthrough Utilization", sbEfficiencyTooltip, eff)}
  {/if}
{/snippet}

{#snippet ssEfficiencyTooltip(val: number)}
  <div class="flex max-w-[400px] flex-col gap-2 text-left">
    <span class="text-sm">
      Represents how much of the total possible benefit gained by the Standing Striker Ark Passive node was actually
      utilized. A value of 0% means that you never had any Standing Striker stacks, while a value of 100% means that you
      had 6 stacks of Standing Striker for the entire fight.
    </span>
    {#if val !== -1}
      <span class="text-sm">
        Your total benefits from Standing Striker can be calculated by multiplying the utilization by the amount
        received at maximum stacks. For your utilization of {(val * 100).toFixed(0)}%, expected benefits are:
      </span>
      <div>
        <div class="text-sm">
          <b>Standing Striker Lv. 1</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(6 + val * (0.75 * 6)).toFixed(1)}% evolution-type damage</li>
            <li>+{(4 + val * (1 * 6)).toFixed(1)}% brand power</li>
          </ul>
        </div>
        <div class="text-sm">
          <b>Standing Striker Lv. 2</b>
          <ul class="list-disc pl-5 text-sm">
            <li>+{(12 + val * (1.5 * 6)).toFixed(1)}% evolution-type damage</li>
            <li>+{(8 + val * (2 * 6)).toFixed(1)}% brand power</li>
          </ul>
        </div>
      </div>
    {:else}
      <span class="text-sm">
        This character does not have Standing Striker equipped, so no utilization was calculated.
      </span>
    {/if}
  </div>
{/snippet}

{#snippet ssEfficiency()}
  {@const icon = EFTable_ArkPassive[1040400]![1]}
  {@const v = split.standingStrikerEfficiency!}
  {#if !v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Standing Striker Utilization", ssEfficiencyTooltip, -1)}
  {:else}
    {@const eff = v.damageDoneByStat / v.damageDoneByStatPlusValue}
    {@render efficiencySection(icon, "Standing Striker Utilization", ssEfficiencyTooltip, eff)}
  {/if}
{/snippet}

<Card class="mt-4">
  <div class="flex items-center justify-between bg-black/10 px-3 py-2 font-medium">
    <div>Stat Conversion Utilization</div>
  </div>

  <div class="grid h-full grid-rows-4">
    {@render rcEfficiency()}
    {@render btEfficiency()}
    {@render sbEfficiency()}
    {@render ssEfficiency()}
  </div>
</Card>

<style lang="postcss">
  .efficiency-block:not(:last-child) {
    border-bottom-width: 1px;
  }
</style>
