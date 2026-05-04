<script lang="ts">
  import { chartable, defaultOptions, type EChartsOptions } from "$lib/charts";
  import { statDataDescriptors, type StatDataGroup } from "$lib/constants/rdps-stats";
  import type { EncounterState } from "$lib/encounter.svelte";
  import type { Entity } from "$lib/types";
  import { abbreviateNumber, getClassIcon } from "$lib/utils";
  import { settings } from "$lib/stores.svelte";
  import { EFTable_ArkPassive } from "$lib/constants/EFTable_ArkPassive";
  import { EFTable_Skill } from "$lib/constants/EFTable_Skill";
  import { Engrave } from "$lib/constants/Engrave";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { IconInfo } from "$lib/icons";
  import Card from "$lib/components/Card.svelte";


  interface Props {
    player: Entity;
    enc: EncounterState;
  }

  interface ContributionGroup {
    name: StatDataGroup;
    total: number;
    subgroups: ContributionSubgroup[];
  }

  interface ContributionSubgroup {
    name: string;
    total: number;
    contributions: Contribution[];
  }

  interface Contribution {
    player: string;
    amount: number;
  }

  let { enc, player }: Props = $props();

  function toContributionGroups(map: Record<string, Record<string, number>>): ContributionGroup[] {
    const perGroup: Record<StatDataGroup, ContributionGroup> = {} as any;
    for (const [player, breakdownMap] of Object.entries(map)) {
      for (const [group, amount] of Object.entries(breakdownMap)) {
        let [mainGroup, subGroup] = group.split("/", 2) as [StatDataGroup, string];
        if (subGroup === "blocky_thorn") subGroup = "Blunt Thorn";

        perGroup[mainGroup] ||= {
          name: mainGroup,
          total: 0,
          subgroups: []
        };

        perGroup[mainGroup].total += amount;

        let subgroup = perGroup[mainGroup].subgroups.find((s) => s.name === subGroup);
        if (!subgroup) {
          subgroup = {
            name: subGroup,
            total: 0,
            contributions: []
          };
          perGroup[mainGroup].subgroups.push(subgroup);
        }

        subgroup.contributions.push({
          player,
          amount
        });
        subgroup.total += amount;
      }
    }

    // sort groups by total contribution first
    const groups = Object.values(perGroup).sort((a, b) => b.total - a.total);
    for (const group of groups) {
      // sort contributions by subgroup, then by amount
      group.subgroups.sort((a, b) => b.total - a.total || a.name.localeCompare(b.name));
    }

    return groups;
  }

  const splits = $derived(enc.encounter!.encounterDamageStats.misc!.contributionSplits ?? []);
  const playerSplit = $derived(splits.find((s) => s.name === player.name)!);

  const contributionGroups = $derived(toContributionGroups(playerSplit.damageDoneByEntitySkillGroup));
  const totalContributed = $derived(contributionGroups.reduce((sum, g) => sum + g.total, 0));
  const selfDamageAmount = $derived(player.damageStats.damageDealt - totalContributed);

  const contributionPieOptions: EChartsOptions = $derived.by((): EChartsOptions => {
    return {
      ...defaultOptions,
      tooltip: {
        trigger: "item",
        formatter: ({ marker, name, value, percent }: any) =>
          `${marker} ${name}<br/>${abbreviateNumber(value as number)} (${percent}%)`
      },
      series: [
        {
          name: "Damage Contribution",
          type: "pie",
          radius: ["40%", "80%"],
          label: {
            alignTo: "edge",
            formatter: ({ name, value }: { name: string; value: number }) => {
              const pct = (value / player.damageStats.damageDealt) * 100;
              const formattedVal = abbreviateNumber(value);
              return `${name}\n{time|${formattedVal} · ${pct.toFixed(1)}%}`;
            },
            rich: {
              time: {
                fontSize: 10,
                color: "#999"
              }
            },
            minMargin: 5,
            edgeDistance: 30,
            lineHeight: 18,
            color: "#fff"
          },
          labelLayout(params: any) {
            const isLeft = params.labelRect.x < 100 / 2;
            const points = params.labelLinePoints;
            // extend line all the way until the edge of the label
            points[2][0] = isLeft ? params.labelRect.x : params.labelRect.x + params.labelRect.width;
            return {
              labelLinePoints: points
            };
          },
          data: [
            ...contributionGroups.map((g) => {
              const pct = (g.total / player.damageStats.damageDealt) * 100;
              const descriptor = statDataDescriptors[g.name];
              return {
                name: typeof descriptor.title === "string" ? descriptor.title : descriptor.title(player.spec!),
                value: g.total
              };
            }),
            {
              name: "Own Skill Damage",
              value: selfDamageAmount,
              itemStyle: { color: settings.classColors[player.class] }
            }
          ]
        }
      ]
    };
  });

  // hack
  function guessIcon(name: string) {
    // try skill
    const skillMatch = Object.values(EFTable_Skill).find((s) => s[0] === name);
    if (skillMatch) return `/images/skills/${skillMatch[2]}.png`;

    // try buffs
    for (const buff of Object.values(enc.encounter!.encounterDamageStats.buffs).concat(
      Object.values(enc.encounter!.encounterDamageStats.debuffs)
    )) {
      if (buff.source.name === name) {
        return `/images/skills/${buff.source.icon}`;
      }
    }

    // try ark passive
    const apMatch = Object.values(EFTable_ArkPassive).find((ap) => ap[0] === name);
    if (apMatch) return `https://cdn.ags.lol/icon/${apMatch[1]}.png`;

    // try engravings
    const engravingMatch = Object.values(Engrave).find((e) => e[0] === name);
    if (engravingMatch) return `https://cdn.ags.lol/icon/${engravingMatch[1]}.png`;

    // wtf
    return "/images/skills/unknown.png";
  }

  function playerClass(name: string): number {
    return enc.encounter!.entities[name]?.classId || 0;
  }
</script>

{#snippet subgroupDetails(subgroup: ContributionSubgroup, showPct: boolean)}
  <div class="flex flex-col">
    <div class="flex flex-row items-center gap-1">
      <img src={guessIcon(subgroup.name)} alt={subgroup.name} class="size-4 rounded-md" />
      <div class="text-sm font-semibold">{subgroup.name}</div>
      {#if showPct}
        <div class="ml-auto text-xs font-medium text-neutral-300">
          {((subgroup.total / player.damageStats.damageDealt) * 100).toFixed(1)}%
        </div>
      {/if}
    </div>
    {#each subgroup.contributions as contribution}
      <div class="tree-item flex flex-row items-center gap-1">
        {#if contribution.player === "DarkGrenadeSynergy"}
          <img class="size-3" src="/images/skills/battle_item_01_47.png" alt="Dark Grenade" />
          <span class="text-xs">Dark Grenade</span>
        {:else}
          <img class="size-3" src={getClassIcon(playerClass(contribution.player))} alt={contribution.player} />
          <span class="text-xs">{contribution.player}</span>
        {/if}
        <QuickTooltip tooltip={contribution.amount.toLocaleString()} class="flex items-center">
          <span class="text-xs font-medium leading-none">{abbreviateNumber(contribution.amount)}</span>
        </QuickTooltip>
      </div>
    {/each}
  </div>
{/snippet}

<Card class="mt-4">
  <div class="flex bg-black/10 px-3 py-2 font-medium">
    <div>Damage Breakdown</div>
  </div>
  <div class="flex flex-col gap-2 divide-y divide-neutral-950">
    <div class="flex w-full items-center justify-center py-3">
      <div class="h-[370px] w-full" use:chartable={contributionPieOptions}></div>
    </div>
    <div class="grid grid-cols-4 gap-4 px-4 py-2">
      {#each contributionGroups as group}
        {@const desc = statDataDescriptors[group.name]}
        {@const pct = (group.total / player.damageStats.damageDealt) * 100}
        <div class="flex flex-col gap-1">
          <div class="flex flex-row justify-between gap-2">
            <div class="flex min-w-0 flex-row items-center gap-1 text-sm font-semibold">
              <span class="truncate">{typeof desc.title === "string" ? desc.title : desc.title(player.spec!)}</span>
              <QuickTooltip>
                <IconInfo class="size-4" />
                {#snippet tooltip()}
                  <div class="max-w-[300px]">
                    {typeof desc.help === "string" ? desc.help : desc.help(player.spec!)}
                  </div>
                {/snippet}
              </QuickTooltip>
            </div>
            <div class="text-sm font-medium">{pct.toFixed(1)}%</div>
          </div>
          <div class="flex flex-col gap-1 border-t border-neutral-600 pt-1">
            {#each group.subgroups as subgroup}
              {@render subgroupDetails(subgroup, group.subgroups.length > 1)}
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
</Card>

<style lang="postcss">
  .tree-item {
    position: relative;
    padding-left: 1.25rem;
  }

  /* The "branch": top-half vertical + horizontal arm */
  .tree-item::before {
    content: "";
    position: absolute;
    left: 6px;
    top: 0;
    width: 0.6rem;
    height: 50%;
    border-left: 1px solid #555;
    border-bottom: 1px solid #555;
  }

  /* The "trunk continuation" below the arm — skipped on last child */
  .tree-item:not(:last-child)::after {
    content: "";
    position: absolute;
    left: 6px;
    top: 50%;
    bottom: 0;
    border-left: 1px solid #555;
  }

  /* Round the corner on the last one */
  .tree-item:last-child::before {
    border-bottom-left-radius: 0.3rem;
  }
</style>
