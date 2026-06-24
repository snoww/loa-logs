<script lang="ts">
  import { chartable, type EChartsOptions } from "$lib/charts";
  import Card from "$lib/components/Card.svelte";
  import GameMessage from "$lib/components/GameMessage.svelte";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import Tooltipped from "$lib/components/Tooltipped.svelte";
  import { ArkGridCore } from "$lib/constants/ArkGrid";
  import { BattleItemData } from "$lib/constants/BattleItem";
  import { classesMap as classIdToNiceName } from "$lib/constants/classes";
  import { EFTable_ArkPassive } from "$lib/constants/EFTable_ArkPassive";
  import { EFTable_Item } from "$lib/constants/EFTable_Item";
  import { EFTable_SeasonCardBook } from "$lib/constants/EFTable_SeasonCardBook";
  import { EFTable_Skill } from "$lib/constants/EFTable_Skill";
  import { EFTable_SkillBuff } from "$lib/constants/EFTable_SkillBuff";
  import { Engrave } from "$lib/constants/Engrave";
  import { type StatDataGroup, statDataDescriptors } from "$lib/constants/rdps-stats";
  import {
    getAbilityFeatureOrigin,
    getSkillBuffOrigin,
    normalizeOrigins,
    type StatOrigin
  } from "$lib/constants/StatOrigin";
  import type { EncounterState } from "$lib/encounter.svelte";
  import { IconInfo, IconRefresh } from "$lib/icons";
  import { extractStatSource, type StatSource, StatSourceType } from "$lib/rdps-breakdown";
  import { settings } from "$lib/stores.svelte";
  import { type Entity } from "$lib/types";
  import { abbreviateNumber, getClassIcon } from "$lib/utils";

  interface Props {
    player: Entity;
    enc: EncounterState;
  }

  // bucket -> source -> player
  interface ContributionBucket {
    name: StatDataGroup;
    total: number;
    subgroups: ContributionSubBucket[];
  }

  interface ContributionSubBucket {
    source: StatSource;
    total: number;
    contributions: Contribution[];
  }

  interface Contribution {
    player: string;
    amount: number;
  }

  // source origin -> player -> bucket
  interface SourceGroup {
    origins: StatOrigin[];
    total: number;
    players: SourceGroupPlayer[];
  }

  interface SourceGroupPlayer {
    player: string;
    total: number;
    buckets: SourceBucket[];
  }

  interface SourceBucket {
    name: StatDataGroup;
    amount: number;
  }

  let { enc, player }: Props = $props();

  const defaultPieOptions: EChartsOptions = {
    textStyle: {
      fontFamily: "Inter Variable"
    },
    grid: {
      left: "5%",
      right: "3%",
      bottom: "2%",
      top: "2%",
      containLabel: true
    }
  };

  function toContributionBuckets(map: Record<string, Record<string, number>>): ContributionBucket[] {
    const perGroup: Record<StatDataGroup, ContributionBucket> = {} as any;
    for (const [player, breakdownMap] of Object.entries(map)) {
      for (const [group, amount] of Object.entries(breakdownMap)) {
        let [mainGroup, subGroup] = group.split("/", 2) as [StatDataGroup, string];

        perGroup[mainGroup] ||= {
          name: mainGroup,
          total: 0,
          subgroups: []
        };

        perGroup[mainGroup].total += amount;

        let source = extractStatSource(subGroup)!;
        let subgroup = perGroup[mainGroup].subgroups.find((s) => JSON.stringify(s.source) === JSON.stringify(source)); // hack
        if (!subgroup) {
          subgroup = {
            source,
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
      // sort contributions by amount
      group.subgroups.sort((a, b) => b.total - a.total);
    }

    return groups;
  }

  function toSourceGroups(buckets: ContributionBucket[]): SourceGroup[] {
    // i love this function so much, it's so fun wrangling this shit
    const sources = new Map<
      string,
      {
        origins: StatOrigin[];
        total: number;
        players: Map<
          string,
          {
            total: number;
            buckets: Map<StatDataGroup, number>;
          }
        >;
      }
    >();

    for (const bucket of buckets) {
      for (const sub of bucket.subgroups) {
        const origins = dedupeOriginsBasedOnName(normalizeOrigins(statSourceToOrigins(sub.source)));
        const key = JSON.stringify(origins);
        let srcEntry = sources.get(key);
        if (!srcEntry) {
          srcEntry = { origins, total: 0, players: new Map() };
          sources.set(key, srcEntry);
        }

        for (const { player, amount } of sub.contributions) {
          srcEntry.total += amount;

          let playerEntry = srcEntry.players.get(player);
          if (!playerEntry) {
            playerEntry = { total: 0, buckets: new Map() };
            srcEntry.players.set(player, playerEntry);
          }

          playerEntry.total += amount;
          playerEntry.buckets.set(bucket.name, (playerEntry.buckets.get(bucket.name) ?? 0) + amount);
        }
      }
    }

    return Array.from(sources.values())
      .map((src) => ({
        origins: src.origins,
        total: src.total,
        players: Array.from(src.players.entries()).map(([player, p]) => ({
          player,
          total: p.total,
          buckets: Array.from(p.buckets.entries()).map(([name, amount]) => ({
            name,
            amount
          }))
        }))
      }))
      .sort((a, b) => b.total - a.total);
  }

  const splits = $derived(enc.encounter!.encounterDamageStats.misc!.contributionSplits ?? []);
  const playerSplit = $derived(splits.find((s) => s.name === player.name)!);

  const contributionBuckets = $derived(toContributionBuckets(playerSplit.damageDoneByEntitySkillGroup));
  const sourceGroups = $derived(toSourceGroups(contributionBuckets));

  const totalContributed = $derived(contributionBuckets.reduce((sum, g) => sum + g.total, 0));
  const selfDamageAmount = $derived(player.damageStats.damageDealt - totalContributed);

  const contributionPieOptions: EChartsOptions = $derived.by((): EChartsOptions => {
    let bucketEntries;

    if (breakdownByOrigin) {
      const maxEntries = 16;
      bucketEntries = sourceGroups.slice(0, maxEntries).map((g) => ({
        name: sourceGroupName(g),
        value: g.total
      }));
      const totalRemainder = sourceGroups.slice(maxEntries).reduce((sum, g) => sum + g.total, 0);
      if (totalRemainder > 0) {
        bucketEntries.push({
          name: "Other Sources",
          value: totalRemainder
        });
      }
    } else {
      bucketEntries = contributionBuckets.map((g) => {
        const descriptor = statDataDescriptors[g.name];
        return {
          name: typeof descriptor.title === "string" ? descriptor.title : descriptor.title(player.spec!),
          value: g.total
        };
      });
    }

    return {
      ...defaultPieOptions,
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
            ...bucketEntries,
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

  let breakdownByOrigin = $state(true);

  // icon of the StatSource; not necessarily the icon of the source of the StatSource
  function statSourceIcon(src: StatSource): string {
    if (src[0] === StatSourceType.Composite) {
      // use original source for composite types
      return statSourceIcon(src[2]);
    }

    if (src[0] === StatSourceType.SkillBuff) {
      const buff = EFTable_SkillBuff[+src[1]];
      return buff ? `https://cdn.ags.lol/icon/${buff[2]}.png` : "/images/skills/unknown.png";
    }

    if (src[0] === StatSourceType.Ability) {
      const ability = Engrave[+src[1]];
      return ability ? `https://cdn.ags.lol/icon/${ability[1]}.png` : "/images/skills/unknown.png";
    }

    if (src[0] === StatSourceType.AbilityFeature) {
      if (src[1] === "blocky_thorn") return `https://cdn.ags.lol/icon/${EFTable_ArkPassive[1040100]![1]}.png`; // Blunt Thorn
    }

    return "/images/skills/unknown.png";
  }

  // name of the StatSource; not necessarily the name of the source of the StatSource
  function statSourceName(src: StatSource): string {
    if (src[0] === StatSourceType.Composite) {
      // use original source for composite types
      return statSourceName(src[2]);
    }

    if (src[0] === StatSourceType.SkillBuff) {
      const buff = EFTable_SkillBuff[+src[1]];
      return buff ? buff[0] : "Untitled Skill Buff";
    }

    if (src[0] === StatSourceType.Ability) {
      const ability = Engrave[+src[1]];
      return ability ? ability[0] : "Unknown Ability";
    }

    if (src[0] === StatSourceType.AbilityFeature) {
      if (src[1].startsWith("ap_identity_holyknight_female_radiant")) return "Shining Knight Crit Conversion";
      if (src[1] === "ap_identity_warlord_lonely_knight") return "Gunlance Skill Crit Conversion";
      if (src[1].startsWith("ap_identity_destroyer")) return "Destroyer State Crit Conversion";
      if (src[1] === "blocky_thorn") return "Blunt Thorn";
      return "UNKNOWN: Please Report!";
    }

    return "Unknown, Please Report! (" + src[0] + ")";
  }

  // resolve a StatSource to a list of origins
  function statSourceToOrigins(src: StatSource): StatOrigin[] {
    if (src[0] === StatSourceType.Composite) {
      // for composite types, use the original source
      return statSourceToOrigins(src[2]);
    }

    if (src[0] === StatSourceType.SkillBuff) {
      return getSkillBuffOrigin(+src[1]);
    }

    if (src[0] === StatSourceType.AbilityFeature) {
      return getAbilityFeatureOrigin(src[1].split(".")[0]);
    }

    return [];
  }

  function statOriginIcon(origin: StatOrigin): string {
    // skill
    if (origin.t === "s") {
      const skill = EFTable_Skill[origin.i];
      return skill && skill[2] !== "_0" ? `https://cdn.ags.lol/icon/${skill[2]}.png` : "/images/skills/unknown.png";
    }

    // bracelet
    if (origin.t === "b") {
      const item = EFTable_Item[213400083];
      return item ? `https://cdn.ags.lol/icon/${item[0]}.png` : "/images/skills/unknown.png";
    }

    // card
    if (origin.t === "c") {
      return "https://cdn.ags.lol/ui/cardmanaicon_on.png";
    }

    // ark grid
    if (origin.t === "ag") {
      const item = EFTable_Item[673000003];
      return item ? `https://cdn.ags.lol/icon/${item[0]}.png` : "/images/skills/unknown.png";
    }

    // ark passive
    if (origin.t === "ap") {
      const passive = EFTable_ArkPassive[origin.i];
      return passive ? `https://cdn.ags.lol/icon/${passive[1]}.png` : "/images/skills/unknown.png";
    }

    // ability
    if (origin.t === "a") {
      const ability = Engrave[origin.i];
      return ability ? `https://cdn.ags.lol/icon/${ability[1]}.png` : "/images/skills/unknown.png";
    }

    // item
    if (origin.t === "it") {
      const item = BattleItemData[origin.i];
      return item ? `https://cdn.ags.lol/icon/${item[1]}.png` : "/images/skills/unknown.png";
    }

    // guardian raid npc
    if (origin.t === "gr") {
      return "/images/icons/boss.png";
    }

    return "/images/skills/unknown.png";
  }

  function statOriginName(origin: StatOrigin): string {
    // skill
    if (origin.t === "s") {
      const skill = EFTable_Skill[origin.i];
      return skill ? skill[0] : "Unknown Skill";
    }

    // bracelet
    if (origin.t === "b") {
      return "Bracelet";
    }

    // card
    if (origin.t === "c") {
      const card = EFTable_SeasonCardBook.find((row) => row[0] === origin.i);
      return card ? card[1] : "Unknown Card Set";
    }

    // ark grid
    if (origin.t === "ag") {
      const core = ArkGridCore[origin.i];
      return core ? core[0] : "Unknown Ark Grid Core";
    }

    // ark passive
    if (origin.t === "ap") {
      const passive = EFTable_ArkPassive[origin.i];
      return passive ? passive[0] : "Unknown Ark Passive";
    }

    // ability
    if (origin.t === "a") {
      const ability = Engrave[origin.i];
      return ability ? ability[0] : "Unknown Ability";
    }

    // item
    if (origin.t === "it") {
      const item = BattleItemData[origin.i];
      return item ? item[0] : "Unknown Item";
    }

    // guardian raid npc
    if (origin.t === "gr") {
      return "Guardian Raid Mechanic";
    }

    return "Unknown, Please Report! (" + JSON.stringify(origin) + ")";
  }

  function statOriginType(origin: StatOrigin): string {
    if (origin.t === "s") {
      const skill = EFTable_Skill[origin.i];
      if (!skill || !skill[1]) return "Skill";
      return classIdToNiceName[skill[1]] + " Skill";
    }
    if (origin.t === "b") return "Bracelet Effect";
    if (origin.t === "c") return "Card Set Effect";
    if (origin.t === "ag") {
      const core = ArkGridCore[origin.i];
      if (!core || (core[1] === 0 && !core[6])) return "Ark Grid Core";
      if (core[1] === 1) return "Chaos Ark Grid Core";
      return `${classIdToNiceName[core[6]]} Order Ark Grid Core`;
    }
    if (origin.t === "ap") {
      const ap = EFTable_ArkPassive[origin.i];
      if (!ap) return "Ark Passive Node";
      const tree = ["Evolution", "Enlightenment", "Leap"][ap[2]];
      if (!ap[6]) return `${tree} Ark Passive Node`;
      return `${classIdToNiceName[ap[6]]} ${tree} Ark Passive Node`;
    }
    if (origin.t === "a") return "Engraving";
    if (origin.t === "it") return "Battle Item";
    if (origin.t === "gr") return "Guardian Raids";
    return "Unknown";
  }

  function sourceGroupName(group: SourceGroup): string {
    const [origin] = group.origins;
    return origin ? statOriginName(origin) : "Unknown Source, Please Report!";
  }

  function dedupeOriginsBasedOnName(origins: StatOrigin[]): StatOrigin[] {
    const seen = new Set<string>();
    const deduped: StatOrigin[] = [];
    for (const origin of origins) {
      const name = statOriginName(origin);
      if (!seen.has(name)) {
        seen.add(name);
        deduped.push(origin);
      }
    }
    return deduped;
  }

  // do we not have origins for this buff, but it's only caused by a boss?
  function isGroupUnknownAndBossOnly(group: SourceGroup): boolean {
    if (group.origins.length !== 0) return false;
    if (group.players.length !== 1) return false;
    const playerEntity = enc.encounter!.entities[group.players[0].player];
    return !!playerEntity && !playerEntity.classId;
  }
</script>

{#snippet playerClassOrBoss(name: string, clazz: string)}
  {@const playerEntity = enc.encounter!.entities[name]}
  {#if !playerEntity}
    <img src="/images/skills/unknown.png" alt="Unknown Entity" class={clazz} />
  {:else if playerEntity.classId}
    <img src={getClassIcon(playerEntity.classId)} alt={playerEntity.class} class={clazz} />
  {:else}
    <img src="/images/icons/boss.png" alt="Boss" class={clazz} />
  {/if}
{/snippet}

{#snippet contributionSubgroupDetails(subgroup: ContributionSubBucket, showPct: boolean)}
  <div class="flex flex-col">
    <div class="flex flex-row items-center gap-1">
      <img src={statSourceIcon(subgroup.source)} alt={statSourceName(subgroup.source)} class="size-4 rounded-md" />
      {#if subgroup.source[0] === StatSourceType.SkillBuff}
        <Tooltipped>
          {#snippet tooltip()}
            {@render sourceBuffTooltip(+subgroup.source[1]!)}
          {/snippet}
          <div class="cursor-default text-sm font-semibold">{statSourceName(subgroup.source)}</div>
        </Tooltipped>
      {:else}
        <div class="text-sm font-semibold">{statSourceName(subgroup.source)}</div>
      {/if}
      {#if subgroup.source[0] === StatSourceType.Composite}
        {@const converter = subgroup.source[1]}
        <QuickTooltip>
          {#snippet tooltip()}
            <div class="flex items-center gap-1">
              <span class="text-neutral-400">This effect got converted due to the use of</span>
              <img src={statSourceIcon(converter)} alt={statSourceName(converter)} class="size-4 rounded-sm" />
              <span>{statSourceName(converter)}</span>
            </div>
          {/snippet}
          <IconRefresh class="ml-1 size-3 text-neutral-400" />
        </QuickTooltip>
      {/if}
      {#if showPct}
        <div class="ml-auto text-xs font-medium text-neutral-300">
          {((subgroup.total / player.damageStats.damageDealt) * 100).toFixed(1)}%
        </div>
      {/if}
    </div>

    {#each subgroup.contributions as contribution}
      <div class="tree-item flex flex-row items-center gap-1">
        {#if contribution.player === "DarkGrenadeSynergy"}
          <img src="https://cdn.ags.lol/icon/{BattleItemData[101151]![1]}.png" alt="Dark Grenade" class="size-3" />
          <span class="text-xs">Dark Grenade</span>
        {:else}
          {@render playerClassOrBoss(contribution.player, "size-3")}
          <span class="text-xs">{contribution.player}</span>
        {/if}
        <QuickTooltip tooltip={contribution.amount.toLocaleString()} class="flex items-center">
          <span class="text-xs leading-none font-medium">{abbreviateNumber(contribution.amount)}</span>
        </QuickTooltip>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet originPlayerDetails(group: SourceGroupPlayer, showPct: boolean)}
  <div class="flex flex-col">
    <div class="flex flex-row items-center gap-1">
      {#if group.player === "DarkGrenadeSynergy"}
        <img
          src="https://cdn.ags.lol/icon/{BattleItemData[101151]![1]}.png"
          alt="Dark Grenade"
          class="size-4 rounded-md"
        />
        <div class="text-sm font-semibold">Dark Grenade</div>
      {:else}
        {@render playerClassOrBoss(group.player, "size-4 rounded-md")}
        <div class="text-sm font-semibold">{group.player}</div>
      {/if}

      {#if showPct}
        <div class="ml-auto text-xs font-medium text-neutral-300">
          {((group.total / player.damageStats.damageDealt) * 100).toFixed(1)}%
        </div>
      {/if}
    </div>
    {#each group.buckets as bucket}
      {@const desc = statDataDescriptors[bucket.name]}
      <div class="tree-item flex flex-row items-center gap-1">
        <span class="text-xs">{typeof desc.title === "string" ? desc.title : desc.title(player.spec!)}</span>
        <QuickTooltip tooltip={bucket.amount.toLocaleString()} class="flex items-center">
          <span class="text-xs leading-none font-medium">{abbreviateNumber(bucket.amount)}</span>
        </QuickTooltip>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet originList(origins: StatOrigin[])}
  {#each origins as origin}
    <div class="flex flex-row items-center gap-2">
      <img src={statOriginIcon(origin)} alt={statOriginName(origin)} class="size-8 rounded-sm" />
      <div class="flex flex-col gap-px">
        <span class="text-sm">{statOriginName(origin)}</span>
        <span class="text-xs text-neutral-400">{statOriginType(origin)}</span>
      </div>
    </div>
  {/each}
{/snippet}

{#snippet byOriginHeaderTooltip(origins: StatOrigin[])}
  <div class="flex max-w-[300px] flex-col gap-2 text-left">
    {#if origins.length > 1}
      <span class="text-xs text-neutral-500">
        Effects in this category are triggered from multiple sources, including:
      </span>
    {/if}
    {@render originList(origins)}
  </div>
{/snippet}

{#snippet sourceBuffTooltip(buffId: number)}
  {@const buff = EFTable_SkillBuff[buffId]}
  {@const origins = dedupeOriginsBasedOnName(getSkillBuffOrigin(buffId))}
  <div class="flex w-[300px] flex-col gap-2 text-left">
    <div class="flex flex-col gap-1">
      <div class="flex flex-row items-center gap-2">
        <img src="https://cdn.ags.lol/icon/{buff[2]}.png" alt={buff[0]} class="size-8 rounded-sm" />
        <div class="flex flex-col gap-px">
          <span class="text-sm">{buff[0]}</span>
          <span class="text-xs text-neutral-400">Status Effect</span>
        </div>
      </div>

      <div class="block text-sm">
        <GameMessage message={buff[1]} scaledVars={{ SkillBuff_sk: "range" }} />
      </div>
    </div>

    {#if origins.length > 0}
      <div class="mt-1 w-full border-b border-neutral-600">
        <span class="text-xs text-neutral-400">This buff is applied by:</span>
      </div>

      {@render originList(origins)}
    {/if}
  </div>
{/snippet}

<Card class="mt-4">
  <div class="flex items-center justify-between bg-black/10 px-3 py-2 font-medium">
    <div>Damage Breakdown</div>
    <div class="flex flex-row">
      <button
        class="rounded-l-lg px-2 py-1 text-sm text-nowrap text-white transition focus:outline-hidden {breakdownByOrigin
          ? 'bg-accent-600 hover:opacity-85'
          : 'bg-neutral-700 hover:bg-neutral-600/60'}"
        onclick={() => {
          breakdownByOrigin = true;
        }}
      >
        By Source
      </button>
      <button
        class="rounded-r-lg px-2 py-1 text-sm text-nowrap text-white transition focus:outline-hidden {!breakdownByOrigin
          ? 'bg-accent-600 hover:opacity-85'
          : 'bg-neutral-700 hover:bg-neutral-600/60'}"
        onclick={() => {
          breakdownByOrigin = false;
        }}
      >
        By Category
      </button>
    </div>
  </div>

  <div class="flex flex-col gap-2 divide-y divide-neutral-950">
    <div class="flex w-full items-center justify-center py-3">
      <div class="h-[400px] w-full" use:chartable={contributionPieOptions}></div>
    </div>

    {#if !breakdownByOrigin}
      <div class="grid grid-cols-4 gap-4 px-4 py-2">
        {#each contributionBuckets as group}
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
                {@render contributionSubgroupDetails(subgroup, group.subgroups.length > 1)}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="grid grid-cols-4 gap-4 px-4 py-2">
        {#each sourceGroups as group}
          {@const pct = (group.total / player.damageStats.damageDealt) * 100}
          {@const unknownAndBossOnly = isGroupUnknownAndBossOnly(group)}
          <div class="flex flex-col gap-1">
            <div class="flex flex-row justify-between gap-2">
              <Tooltipped
                class="flex min-w-0 cursor-default flex-row items-center gap-1 align-baseline text-sm font-semibold"
              >
                {#snippet tooltip()}
                  {@render byOriginHeaderTooltip(group.origins)}
                {/snippet}

                {#if group.origins.length > 0}
                  <img src={statOriginIcon(group.origins[0])} alt="" class="size-4 rounded-sm" />
                  <span class="truncate">{statOriginName(group.origins[0])}</span>
                  {#if group.origins.length > 1}
                    <span class="border-b border-dashed text-xs text-neutral-300">+{group.origins.length - 1}</span>
                  {/if}
                {:else if unknownAndBossOnly}
                  <img src="/images/icons/boss.png" alt="Raid Mechanics" class="size-4 rounded-sm" />
                  <span class="truncate">Raid Mechanics</span>
                {:else}
                  <img src="/images/skills/unknown.png" alt="Unknown Source" class="size-4 rounded-sm" />
                  <span class="truncate">Unknown Source, Please Report!</span>
                {/if}
              </Tooltipped>
              <div class="text-sm font-medium">{pct.toFixed(1)}%</div>
            </div>
            <div class="flex flex-col gap-1 border-t border-neutral-600 pt-1">
              {#each group.players as player}
                {@render originPlayerDetails(player, group.players.length > 1)}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
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
