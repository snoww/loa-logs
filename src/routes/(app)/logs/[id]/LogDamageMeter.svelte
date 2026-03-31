<script lang="ts">
  import { chartable, type EChartsOptions } from "$lib/charts";
  import ArcanistCardTable from "$lib/components/ArcanistCardTable.svelte";
  import BossBreakdown from "$lib/components/BossBreakdown.svelte";
  import BossTable from "$lib/components/BossTable.svelte";
  import Buffs from "$lib/components/Buffs.svelte";
  import Card from "$lib/components/Card.svelte";
  import DamageMeterPartySplit from "$lib/components/DamageMeterPartySplit.svelte";
  import DamageTaken from "$lib/components/DamageTaken.svelte";
  import LogPlayerBreakdown from "$lib/components/PlayerBreakdown.svelte";
  import { EncounterState } from "$lib/encounter.svelte.js";
  import { screenshot } from "$lib/stores.svelte.js";
  import { bossHpMap } from "$lib/constants/encounters";
  import { BossHpLog, ChartType, type Encounter, type Entity, EntityType, MeterState, MeterTab } from "$lib/types";
  import { resampleData } from "$lib/utils";
  import {
    getAllDeathInfo,
    getAverageDpsChart,
    getAveragePlayerSeries,
    getBasicSkillLogChart,
    getBossHpSeries,
    getDetailedSkillLogChart,
    getLegendNames,
    getPlayerIncapSeries,
    getRollingDpsChart,
    getRollingPlayerSeries
  } from "$lib/utils/dpsCharts";
  import { getSupportSynergiesOverTime, getSupportSynergiesOverTimeChart } from "$lib/utils/supportBuffCharts";
  import { onDestroy } from "svelte";
  import LogEncounterInfo from "./LogEncounterInfo.svelte";
  import LogQuickControls from "./LogQuickControls.svelte";
  import LogQuickSettings from "./LogQuickSettings.svelte";
  import LogScreenshotInfo from "./LogScreenshotInfo.svelte";
  import LogShields from "./LogShields.svelte";
  import LogSkillDetails from "./LogSkillDetails.svelte";
  import OpenerSkills from "./OpenerSkills.svelte";

  let { encounter }: { encounter: Encounter } = $props();

  let enc = $derived(new EncounterState(encounter, false));

  let meterState = $state(MeterState.PARTY);
  let tab = $state(MeterTab.DAMAGE);
  let chartType = $state(ChartType.AVERAGE_DPS);
  let timeWindow: { startMs: number; endMs: number } | null = $state(null);
  let echartsInst: ReturnType<typeof chartable>["echartsInstance"] | null = $state(null);

  let playerName = $state("");
  let player: Entity | undefined = $derived.by(() => {
    if (playerName) {
      return encounter.entities[playerName];
    }
  });

  let focusedBoss = $state("");

  $effect(() => {
    encounter.fightStart;
    playerName = "";
    focusedBoss = "";
    meterState = MeterState.PARTY;
    chartType = ChartType.AVERAGE_DPS;
    timeWindow = null;
  });

  $effect(() => {
    enc.timeWindow = timeWindow;
  });

  function inspectPlayer(name: string) {
    meterState = MeterState.PLAYER;
    chartType = ChartType.SKILL_LOG;
    playerName = name;
    scrollToTop();
  }

  function inspectBoss(name: string) {
    meterState = MeterState.PLAYER;
    chartType = ChartType.SKILL_LOG;
    focusedBoss = name;
    scrollToTop();
  }

  const hasSkillDetails = $derived(
    Object.values(encounter.entities).some((e) =>
      Object.values(e.skills).some((skill) => skill.skillCastLog?.length > 0)
    )
  );

  let chartOptions: EChartsOptions = $state({});
  let chartablePlayers = $derived(
    Object.values(encounter.entities)
      .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId !== 0)
      .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt)
  );
  let bossHpLogs = $derived(Object.entries(encounter.encounterDamageStats.bossHpLog || {}));
  let legendNames = $derived(getLegendNames(chartablePlayers));
  let aggregateDps = $derived.by(() => {
    if (chartablePlayers.length === 0) return [];
    const len = chartablePlayers[0].damageStats.dpsAverage.length;
    const result = new Array(len).fill(0);
    for (const p of chartablePlayers) {
      const arr = p.damageStats.dpsAverage;
      for (let i = 0; i < Math.min(arr.length, len); i++) {
        result[i] += arr[i];
      }
    }
    return result;
  });
  let bossHpResampled = $derived.by(() => {
    const len = aggregateDps.length;
    if (len === 0 || bossHpLogs.length === 0) return [];
    const primary = bossHpLogs.reduce((a, b) => (a[1].length >= b[1].length ? a : b));
    const bossName = primary[0];
    const log = primary[1];
    if (log.length < 2) return [];
    const max = Math.max(...log.slice(0, 5).map((e) => e.p));
    const normalized = max > 1 ? log.map((e) => new BossHpLog(e.time, e.hp, e.p / max)) : log;
    const resampled = resampleData(normalized, 5, len);
    const totalBars = bossHpMap[bossName] ?? 0;
    return resampled.map((e) => ({ bars: totalBars > 0 ? Math.round(e.p * totalBars) : 0, p: e.p }));
  });
  let buffChartLegend = $derived(enc.parties.map((_, i) => "Party " + (i + 1)));
  let buffChartSeries = $derived(
    getSupportSynergiesOverTime(enc, encounter.fightStart, encounter.lastCombatPacket, 5000)
  );
  let buffChartBosses = $derived(getBossHpSeries(bossHpLogs, buffChartLegend, buffChartSeries[0].data.length, 5));

  let chartDiv: HTMLElement | null = $state(null);
  let destroyChart: (() => void) | null = $state(null);

  $effect(() => {
    if (chartDiv) {
      const { destroy, echartsInstance } = chartable(chartDiv, chartOptions);
      destroyChart = destroy;
      echartsInst = echartsInstance;
    }
  });

  function commitTimeWindow(w: { startSec: number; endSec: number } | null) {
    timeWindow = w ? { startMs: w.startSec * 1000, endMs: w.endSec * 1000 } : null;
    applyWindowToCharts(w?.startSec ?? 0, w?.endSec ?? encounter.duration / 1000);
  }

  function applyWindowToCharts(startSec: number, endSec: number) {
    const durSec = encounter.duration / 1000;
    if (durSec <= 0) return;
    echartsInst?.dispatchAction({
      type: "dataZoom",
      start: (startSec / durSec) * 100,
      end: (endSec / durSec) * 100
    });
  }

  onDestroy(() => {
    if (destroyChart) destroyChart();
  });

  $effect(() => {
    if (
      chartablePlayers.length > 0 &&
      chartablePlayers[0]!.damageStats &&
      chartablePlayers[0]!.damageStats.dpsAverage.length > 0 &&
      chartablePlayers[0]!.damageStats.dpsRolling10sAvg.length > 0
    ) {
      let deathInfo = getAllDeathInfo(chartablePlayers, legendNames, encounter.fightStart);
      if (chartType === ChartType.AVERAGE_DPS) {
        let chartPlayers = getAveragePlayerSeries(chartablePlayers, legendNames, encounter.fightStart);
        let bossChart = getBossHpSeries(bossHpLogs, legendNames, chartablePlayers[0].damageStats.dpsAverage.length, 5);
        chartOptions = getAverageDpsChart(chartablePlayers, legendNames, chartPlayers, bossChart, deathInfo);
      } else if (chartType === ChartType.ROLLING_DPS) {
        let chartPlayers = getRollingPlayerSeries(chartablePlayers, legendNames, encounter.fightStart);
        let bossChart = getBossHpSeries(
          bossHpLogs,
          legendNames,
          chartablePlayers[0].damageStats.dpsRolling10sAvg.length,
          1
        );
        chartOptions = getRollingDpsChart(chartablePlayers, legendNames, chartPlayers, bossChart, deathInfo);
      } else if (chartType === ChartType.SKILL_LOG && player && player.entityType === EntityType.PLAYER) {
        const skillLogLen = Math.floor((encounter.lastCombatPacket - encounter.fightStart) / 1000);
        const skillLogLegend: string[] = [];
        const skillLogBosses = [
          ...getBossHpSeries(bossHpLogs, skillLogLegend, skillLogLen, 1),
          ...getPlayerIncapSeries(player, encounter.fightStart, encounter.lastCombatPacket)
        ];
        if (hasSkillDetails) {
          chartOptions = getDetailedSkillLogChart(
            player,
            encounter.lastCombatPacket,
            encounter.fightStart,
            encounter.encounterDamageStats,
            skillLogBosses
          );
        } else {
          chartOptions = getBasicSkillLogChart(
            player,
            encounter.lastCombatPacket,
            encounter.fightStart,
            skillLogBosses
          );
        }
      } else if (chartType === ChartType.BRAND_BUFF) {
        chartOptions = getSupportSynergiesOverTimeChart(buffChartLegend, buffChartSeries, "_1_", buffChartBosses);
      } else if (chartType === ChartType.AP_BUFF) {
        chartOptions = getSupportSynergiesOverTimeChart(buffChartLegend, buffChartSeries, "_0_", buffChartBosses);
      } else if (chartType === ChartType.IDENTITY) {
        chartOptions = getSupportSynergiesOverTimeChart(buffChartLegend, buffChartSeries, "_2_", buffChartBosses);
      } else if (chartType === ChartType.HYPER_BUFF) {
        chartOptions = getSupportSynergiesOverTimeChart(buffChartLegend, buffChartSeries, "_3_", buffChartBosses);
      }
    }
  });

  function setChartView() {
    if (meterState === MeterState.PARTY) {
      chartType = ChartType.AVERAGE_DPS;
    } else if (meterState === MeterState.PLAYER) {
      chartType = ChartType.SKILL_LOG;
    }
  }

  function handleRightClick(e?: MouseEvent) {
    if (e) {
      e.preventDefault();
    }
    if (meterState === MeterState.PLAYER) {
      meterState = MeterState.PARTY;
      playerName = "";
      focusedBoss = "";
      chartType = ChartType.AVERAGE_DPS;
      scrollToTop();
    }
  }

  function scrollToTop() {
    window.scrollTo(0, 0);
  }

  let screenshotDiv: HTMLElement | undefined = $state();
</script>

{#snippet logTab(selectedTab: MeterTab, tabName: string)}
  <button
    class="rounded-lg px-2 py-1 text-sm text-nowrap text-white transition focus:outline-hidden {tab === selectedTab
      ? 'bg-accent-500/80'
      : 'hover:bg-neutral-800/40'}"
    onclick={() => {
      if (
        selectedTab === MeterTab.BOSS ||
        selectedTab === MeterTab.TANK ||
        selectedTab === MeterTab.SHIELDS ||
        focusedBoss
      ) {
        handleRightClick();
      }

      tab = selectedTab;
      setChartView();
    }}
  >
    {tabName}
  </button>
{/snippet}

{#snippet chartTab(selectedTab: ChartType, tabName: string, border = false)}
  <button
    class="px-2 py-1 text-sm text-nowrap text-white transition first:rounded-l-lg last:rounded-r-lg focus:outline-hidden {chartType ===
    selectedTab
      ? 'bg-accent-500/80'
      : 'hover:bg-neutral-800/40'} {border ? 'border-l-1 border-neutral-900/80' : ''}
      "
    onclick={() => {
      chartType = selectedTab;
    }}
  >
    {tabName}
  </button>
{/snippet}

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overflow-hidden text-neutral-100" oncontextmenu={handleRightClick}>
  <Card bind:self={screenshotDiv}>
    <!-- Enocunter summary -->
    <LogEncounterInfo {enc} durationSec={Math.floor(encounter.duration / 1000)} dpsData={aggregateDps} bossHpData={bossHpResampled} oncommit={commitTimeWindow} />

    <!-- Content and tabs -->
    <div class="flex flex-col gap-2 py-2">
      <div
        class="mx-2 flex w-full max-w-fit flex-row gap-1 overflow-x-auto rounded-lg bg-neutral-700"
        class:hidden={screenshot.state}
      >
        {@render logTab(MeterTab.DAMAGE, "Damage")}
        {@render logTab(MeterTab.PARTY_BUFFS, "Party Buffs")}
        {@render logTab(MeterTab.SELF_BUFFS, "Self Buffs")}
        {#if encounter.encounterDamageStats.totalShielding > 0}
          {@render logTab(MeterTab.SHIELDS, "Shields")}
        {/if}
        {#if encounter.encounterDamageStats.totalDamageTaken > 0}
          {@render logTab(MeterTab.TANK, "Tanked")}
        {/if}
        {@render logTab(MeterTab.BOSS, "Bosses")}
        <LogQuickControls bind:encounter {screenshotDiv} />
        <LogQuickSettings />
      </div>
      <!-- screenshot info -->
      <LogScreenshotInfo {encounter} />
      <!-- main content -->
      <div
        class="relative top-0 overflow-x-auto overflow-y-visible rounded text-sm md:px-2 {screenshot.state
          ? 'px-3! pb-2!'
          : ''}"
      >
        {#if tab === MeterTab.DAMAGE}
          {#if meterState === MeterState.PARTY}
            <DamageMeterPartySplit {enc} {inspectPlayer} />
          {:else if meterState === MeterState.PLAYER && player !== undefined}
            <table class="relative isolate w-full table-fixed">
              <LogPlayerBreakdown entity={player} {enc} {handleRightClick} />
            </table>
            {#if player.class === "Arcanist"}
              <table class="relative isolate w-full table-fixed">
                <ArcanistCardTable {player} duration={encounter.duration} />
              </table>
            {/if}
          {/if}
        {:else if tab === MeterTab.PARTY_BUFFS || tab === MeterTab.SELF_BUFFS}
          <Buffs {tab} {enc} focusedPlayer={player} {inspectPlayer} {handleRightClick} />
        {:else if tab === MeterTab.TANK}
          <DamageTaken {enc} />
        {:else if tab === MeterTab.SHIELDS}
          <LogShields {enc} />
        {:else if tab === MeterTab.BOSS}
          {#if !focusedBoss}
            <BossTable {enc} {inspectBoss} />
          {:else}
            <BossBreakdown {enc} boss={encounter.entities[focusedBoss]} handleRightClick={() => (focusedBoss = "")} />
          {/if}
        {/if}
      </div>
    </div>
  </Card>

  <!-- Opener skills -->
  {#if chartType === ChartType.SKILL_LOG && player && player.entityType === EntityType.PLAYER}
    <OpenerSkills skills={player.skills} />
  {/if}

  <!-- Charts -->
  {#if player?.entityType !== EntityType.ESTHER}
    <Card class="mt-4">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="bg-black/10 px-3 py-2 font-medium">Charts</div>
      <div class="py-2" oncontextmenu={handleRightClick}>
        <div class="mx-2 flex w-fit overflow-x-auto rounded-lg bg-neutral-700 max-md:max-w-[100vw]">
          {#if playerName === "" && meterState === MeterState.PARTY}
            {@render chartTab(ChartType.AVERAGE_DPS, "Average DPS")}
            {@render chartTab(ChartType.ROLLING_DPS, "10s DPS Window")}
            {#if hasSkillDetails}
              {#if enc.anySupportBuff}
                {@render chartTab(ChartType.AP_BUFF, "AP Buffs", true)}
              {/if}
              {#if enc.anySupportBrand}
                {@render chartTab(ChartType.BRAND_BUFF, "Brand")}
              {/if}
              {#if enc.anySupportIdentity}
                {@render chartTab(ChartType.IDENTITY, "Identity")}
              {/if}
              {#if enc.anySupportHat}
                {@render chartTab(ChartType.HYPER_BUFF, "H.A Skill")}
              {/if}
            {/if}
          {/if}
        </div>
        <div class="mt-2 h-[300px] w-full" use:chartable={chartOptions} bind:this={chartDiv}></div>
      </div>
    </Card>
  {/if}
  <!-- Skill cast details -->
  {#if hasSkillDetails && player}
    <LogSkillDetails player={player!} encounterDamageStats={encounter.encounterDamageStats} />
  {/if}
</div>
