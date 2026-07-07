<script lang="ts">
  import { resolve } from "$app/paths";
  import { getRaidProgressionRange, getRaidProgressionStatistics } from "$lib/api";
  import DateRangePicker from "$lib/components/DateRangePicker.svelte";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { difficultyColor } from "$lib/components/Snippets.svelte";
  import { difficultyMap, encounterMap } from "$lib/constants/encounters";
  import { IconArrowUp, IconRotateCcw } from "$lib/icons";
  import type { RaidProgressionPlayer, RaidProgressionPull, RaidProgressionStatistics } from "$lib/types";
  import { getClassIcon } from "$lib/utils";
  import { onMount } from "svelte";

  import {
    dateToEndTime,
    dateToStartTime,
    formatDateTime,
    formatDps,
    formatDuration,
    formatNumber,
    formatPercent,
    formatRatioPercent,
    formatShortDate as formatDate,
    formatTotalDuration,
    timestampToInputDate
  } from "../format";

  type GateOption = {
    id: string;
    raid: string;
    gate: string;
    bosses: string[];
  };

  type ProgressionFilters = {
    selectedGateId: string;
    selectedDifficulty: string;
    startDate: string;
    endDate: string;
  };

  type ProgressionPageCache = {
    url: string;
    selectedGateId: string;
    selectedDifficulty: string;
    startDate: string;
    endDate: string;
    defaultStartDate: string;
    defaultEndDate: string;
    statistics: RaidProgressionStatistics | null;
    hasLoaded: boolean;
    appliedFilters?: ProgressionFilters | null;
    error: string;
  };

  type ProgressionPlayerSortKey =
    | "averageDps"
    | "averageNdps"
    | "averageRdps"
    | "averageDamageTaken"
    | "deathsPerPull"
    | "totalDeaths"
    | "averageSupportAp"
    | "averageSupportBrand"
    | "averageSupportIdentity"
    | "averageSupportHyper";

  type ProgressionPlayerTable = "dps" | "support";
  type SortDirection = "asc" | "desc";

  type PlayerSortState = {
    key: ProgressionPlayerSortKey;
    direction: SortDirection;
  } | null;

  type DeathRateSummary = {
    names: string[];
    rate: number;
  } | null;

  const sortAccessors: Record<ProgressionPlayerSortKey, (player: RaidProgressionPlayer) => number | null | undefined> =
    {
      averageDps: (player) => player.averageDps,
      averageNdps: (player) => player.averageNdps,
      averageRdps: (player) => player.averageRdps,
      averageDamageTaken: (player) => player.averageDamageTaken,
      deathsPerPull: (player) => player.deathsPerPull,
      totalDeaths: (player) => player.totalDeaths,
      averageSupportAp: (player) => player.averageSupportAp,
      averageSupportBrand: (player) => player.averageSupportBrand,
      averageSupportIdentity: (player) => player.averageSupportIdentity,
      averageSupportHyper: (player) => player.averageSupportHyper
    };

  const gateOptionGroups = Object.entries(encounterMap)
    .reverse()
    .map(([raid, gates]) => ({
      raid,
      gates: Object.entries(gates).map(
        ([gate, bosses]): GateOption => ({
          id: `${raid}:${gate}`,
          raid,
          gate,
          bosses
        })
      )
    }));
  const gateOptions = gateOptionGroups.flatMap((group) => group.gates);
  const defaultGateId = gateOptions[0]?.id ?? "";
  const cacheStorageKey = "loa-logs:raid-progression-state";
  let cachedProgressionState: ProgressionPageCache | null = null;

  let selectedGateId = $state(defaultGateId);
  let selectedDifficulty = $state("");
  let startDate = $state("");
  let endDate = $state("");
  let defaultStartDate = $state("");
  let defaultEndDate = $state("");
  let statistics = $state<RaidProgressionStatistics | null>(null);
  let appliedFilters = $state<ProgressionFilters | null>(null);
  let loading = $state(false);
  let rangeLoading = $state(false);
  let hasLoaded = $state(false);
  let error = $state("");
  let initialized = $state(false);
  let dpsSort = $state<PlayerSortState>(null);
  let supportSort = $state<PlayerSortState>(null);
  let requestId = 0;
  let rangeRequestId = 0;
  let keepAppliedDates = false;

  let selectedGateOption = $derived(gateOptions.find((gate) => gate.id === selectedGateId));
  let selectedBosses = $derived(selectedGateOption?.bosses ?? []);
  let selectedBossOrder = $derived.by(() => Object.fromEntries(selectedBosses.map((boss, index) => [boss, index])));
  let selectedBossToGate = $derived.by(() =>
    Object.fromEntries(selectedBosses.map((boss) => [boss, selectedGateOption?.gate ?? boss]))
  );
  let selectedClearBosses = $derived(selectedBosses);
  let filtersChanged = $derived(
    selectedGateId !== defaultGateId ||
      selectedDifficulty !== "" ||
      startDate !== defaultStartDate ||
      endDate !== defaultEndDate
  );
  let currentFilters = $derived({
    selectedGateId,
    selectedDifficulty,
    startDate,
    endDate
  });
  let hasUnappliedFilters = $derived(
    appliedFilters === null || !sameProgressionFilters(currentFilters, appliedFilters)
  );
  let canLoadStatistics = $derived(
    Boolean(selectedGateOption) && !loading && !rangeLoading && (!hasLoaded || hasUnappliedFilters || Boolean(error))
  );
  let dpsPlayers = $derived((statistics?.players ?? []).filter((player) => !player.isSupport));
  let supportPlayers = $derived((statistics?.players ?? []).filter((player) => player.isSupport));
  let sortedDpsPlayers = $derived(sortPlayers(dpsPlayers, dpsSort));
  let sortedSupportPlayers = $derived(sortPlayers(supportPlayers, supportSort));
  let showSupportContribution = $derived(
    supportPlayers.some(
      (player) => player.averageSupportContribution !== undefined && player.averageSupportContribution !== null
    )
  );
  let leastDeathsPerPull = $derived(deathRateSummary(statistics?.players ?? [], "least"));
  let mostDeathsPerPull = $derived(deathRateSummary(statistics?.players ?? [], "most"));
  let pullRows = $derived(statistics?.pulls ?? []);

  onMount(() => {
    if (restoreCachedProgressionState()) {
      initialized = true;
      return;
    }

    applyUrlState();
    initialized = true;
    const preserveDates = keepAppliedDates;
    keepAppliedDates = false;
    loadDefaultRange(preserveDates);
  });

  $effect(() => {
    if (!initialized) return;

    selectedGateId;
    selectedDifficulty;
    startDate;
    endDate;
    defaultStartDate;
    defaultEndDate;
    statistics;
    hasLoaded;
    appliedFilters;
    error;

    updateUrlState();
    cacheProgressionState();
  });

  async function loadDefaultRange(preserveExistingDates = false) {
    if (!selectedGateOption) {
      defaultStartDate = "";
      defaultEndDate = "";
      startDate = "";
      endDate = "";
      return;
    }

    const currentRequest = ++rangeRequestId;
    rangeLoading = true;
    error = "";

    try {
      const result = await getRaidProgressionRange({
        bosses: selectedBosses,
        lastGateBosses: selectedClearBosses,
        difficulty: selectedDifficulty,
        minDuration: 10
      });

      if (currentRequest === rangeRequestId) {
        const nextStartDate = timestampToInputDate(result.firstPull);
        const nextEndDate = timestampToInputDate(result.firstClear);
        defaultStartDate = nextStartDate;
        defaultEndDate = nextEndDate;
        if (!preserveExistingDates || !startDate) {
          startDate = nextStartDate;
        }
        if (!preserveExistingDates || !endDate) {
          endDate = nextEndDate;
        }
      }
    } catch (err) {
      console.error(err);
      if (currentRequest === rangeRequestId) {
        error = "Could not load progression date range.";
      }
    } finally {
      if (currentRequest === rangeRequestId) {
        rangeLoading = false;
      }
    }
  }

  async function loadStatistics() {
    const currentRequest = ++requestId;
    loading = true;
    error = "";

    try {
      const result = await getRaidProgressionStatistics({
        range: "all",
        bossToRaid: selectedBossToGate,
        bossOrder: selectedBossOrder,
        bosses: selectedBosses,
        lastGateBosses: selectedClearBosses,
        difficulty: selectedDifficulty,
        startTime: dateToStartTime(startDate),
        endTime: dateToEndTime(endDate),
        minDuration: 10
      });

      if (currentRequest === requestId) {
        statistics = result;
        appliedFilters = currentFilters;
        hasLoaded = true;
      }
    } catch (err) {
      console.error(err);
      if (currentRequest === requestId) {
        hasLoaded = true;
        error = "Could not load progression statistics.";
      }
    } finally {
      if (currentRequest === requestId) {
        loading = false;
      }
    }
  }

  function applyUrlState() {
    const params = new URLSearchParams(window.location.search);

    const requestedGate = params.get("gate");
    const requestedRaid = params.get("raid");
    const matchingGate =
      gateOptions.find((gate) => gate.id === requestedGate || gate.gate === requestedGate) ??
      gateOptions.find((gate) => gate.raid === requestedRaid);
    selectedGateId = matchingGate?.id ?? defaultGateId;

    const requestedDifficulty = params.get("difficulty");
    selectedDifficulty = requestedDifficulty && difficultyMap.includes(requestedDifficulty) ? requestedDifficulty : "";

    startDate = params.get("start") ?? "";
    endDate = params.get("end") ?? "";
    keepAppliedDates = Boolean(startDate || endDate);
  }

  function updateUrlState() {
    const params = new URLSearchParams();
    if (selectedGateOption && selectedGateId !== defaultGateId) params.set("gate", selectedGateOption.gate);
    if (selectedDifficulty) params.set("difficulty", selectedDifficulty);
    if (startDate) params.set("start", startDate);
    if (endDate) params.set("end", endDate);

    const query = params.toString();
    const nextUrl = `${window.location.pathname}${query ? `?${query}` : ""}`;
    if (nextUrl !== `${window.location.pathname}${window.location.search}`) {
      window.history.replaceState(window.history.state, "", nextUrl);
    }
  }

  function currentUrlKey() {
    return `${window.location.pathname}${window.location.search}`;
  }

  function cacheProgressionState() {
    const state = {
      url: currentUrlKey(),
      selectedGateId,
      selectedDifficulty,
      startDate,
      endDate,
      defaultStartDate,
      defaultEndDate,
      statistics,
      hasLoaded,
      appliedFilters,
      error
    };
    cachedProgressionState = state;

    try {
      sessionStorage.setItem(cacheStorageKey, JSON.stringify(state));
    } catch (err) {
      console.warn("failed to cache raid progression state", err);
    }
  }

  function restoreCachedProgressionState() {
    const cache = cachedProgressionState ?? readStoredProgressionState();
    if (
      cache === null ||
      cache.url !== currentUrlKey() ||
      !gateOptions.some((gate) => gate.id === cache.selectedGateId)
    ) {
      return false;
    }

    cachedProgressionState = cache;
    selectedGateId = cache.selectedGateId;
    selectedDifficulty = cache.selectedDifficulty;
    startDate = cache.startDate;
    endDate = cache.endDate;
    defaultStartDate = cache.defaultStartDate;
    defaultEndDate = cache.defaultEndDate;
    statistics = cache.statistics;
    hasLoaded = cache.hasLoaded;
    appliedFilters = cache.appliedFilters ?? (cache.hasLoaded ? currentFilters : null);
    error = cache.error;
    loading = false;
    rangeLoading = false;
    keepAppliedDates = false;
    ++requestId;
    ++rangeRequestId;
    return true;
  }

  function readStoredProgressionState() {
    try {
      const stored = sessionStorage.getItem(cacheStorageKey);
      if (!stored) return null;
      return JSON.parse(stored) as ProgressionPageCache;
    } catch (err) {
      console.warn("failed to restore raid progression state", err);
      sessionStorage.removeItem(cacheStorageKey);
      return null;
    }
  }

  function cancelPendingStatisticsLoad() {
    ++requestId;
    loading = false;
  }

  function updateGate(value: string) {
    if (selectedGateId === value) return;
    selectedGateId = value;
    selectedDifficulty = "";
    cancelPendingStatisticsLoad();
    loadDefaultRange();
  }

  function updateDifficulty(value: string) {
    if (selectedDifficulty === value) return;
    selectedDifficulty = value;
    cancelPendingStatisticsLoad();
    loadDefaultRange();
  }

  function updateStartDate(value: string) {
    ++rangeRequestId;
    rangeLoading = false;
    cancelPendingStatisticsLoad();
    startDate = value;
  }

  function updateEndDate(value: string) {
    ++rangeRequestId;
    rangeLoading = false;
    cancelPendingStatisticsLoad();
    endDate = value;
  }

  function resetFilters() {
    cancelPendingStatisticsLoad();
    selectedDifficulty = "";
    startDate = "";
    endDate = "";
    loadDefaultRange();
  }

  function sameProgressionFilters(a: ProgressionFilters, b: ProgressionFilters) {
    return (
      a.selectedGateId === b.selectedGateId &&
      a.selectedDifficulty === b.selectedDifficulty &&
      a.startDate === b.startDate &&
      a.endDate === b.endDate
    );
  }

  function progressLabel(source: { bestProgressBars?: number | null; bestProgressPercent?: number | null }) {
    if (source.bestProgressBars !== undefined && source.bestProgressBars !== null) {
      return `${source.bestProgressBars}x`;
    }
    if (source.bestProgressPercent !== undefined && source.bestProgressPercent !== null) {
      return `${source.bestProgressPercent.toFixed(1)}%`;
    }
    return "-";
  }

  function pullProgressLabel(pull: RaidProgressionPull) {
    if (pull.cleared) return "Clear";
    if (pull.progressBars !== undefined && pull.progressBars !== null) return `${pull.progressBars}x`;
    if (pull.progressPercent !== undefined && pull.progressPercent !== null)
      return `${pull.progressPercent.toFixed(1)}%`;
    return "-";
  }

  function progressCompletePercent(value?: number | null, cleared = false) {
    if (cleared) return 100;
    if (value === undefined || value === null) return 0;
    return Math.max(0, Math.min(100, 100 - value));
  }

  function sortPlayers(players: RaidProgressionPlayer[], sort: PlayerSortState) {
    if (sort === null) return players;

    const direction = sort.direction === "asc" ? 1 : -1;
    const accessor = sortAccessors[sort.key];
    return players.toSorted((a, b) => {
      const aValue = accessor(a) ?? undefined;
      const bValue = accessor(b) ?? undefined;
      if (aValue === undefined && bValue === undefined) return a.name.localeCompare(b.name);
      if (aValue === undefined) return 1;
      if (bValue === undefined) return -1;
      return (aValue - bValue) * direction || a.name.localeCompare(b.name);
    });
  }

  function updatePlayerSort(table: ProgressionPlayerTable, key: ProgressionPlayerSortKey) {
    const currentSort = table === "dps" ? dpsSort : supportSort;
    let nextSort: PlayerSortState = { key, direction: "desc" };

    if (currentSort?.key === key) {
      nextSort = currentSort.direction === "desc" ? { key, direction: "asc" } : null;
    }

    if (table === "dps") {
      dpsSort = nextSort;
    } else {
      supportSort = nextSort;
    }
  }

  function playerSort(table: ProgressionPlayerTable) {
    return table === "dps" ? dpsSort : supportSort;
  }

  function sortAria(
    table: ProgressionPlayerTable,
    key: ProgressionPlayerSortKey
  ): "ascending" | "descending" | undefined {
    const sort = playerSort(table);
    if (sort?.key !== key) return undefined;
    return sort.direction === "asc" ? "ascending" : "descending";
  }

  function isActiveSort(table: ProgressionPlayerTable, key: ProgressionPlayerSortKey) {
    return playerSort(table)?.key === key;
  }

  function deathRateSummary(players: RaidProgressionPlayer[], mode: "least" | "most"): DeathRateSummary {
    const candidates = players.filter((player) => player.pulls > 0);
    if (candidates.length === 0) return null;

    const rate = candidates
      .filter((players) => players.pulls > 1)
      .reduce((selectedRate, player) => {
        if (mode === "least") return Math.min(selectedRate, player.deathsPerPull);
        return Math.max(selectedRate, player.deathsPerPull);
      }, candidates[0]!.deathsPerPull);

    return {
      rate,
      names: candidates
        .filter((player) => player.deathsPerPull === rate)
        .map((player) => player.name)
        .sort((a, b) => a.localeCompare(b))
    };
  }

  function formatDeathRate(summary: DeathRateSummary) {
    if (summary === null) return "-";
    return summary.rate.toFixed(2);
  }

  function deathRateNamesLabel(summary: DeathRateSummary) {
    if (summary === null) return "-";
    if (summary.names.length === 1) return summary.names[0];
    if (summary.names.length <= 3) return summary.names.join(", ");
    return `${summary.names.length} players`;
  }

  function deathRateTooltip(summary: DeathRateSummary) {
    if (summary === null) return "";
    return `${summary.rate.toFixed(2)} deaths/pull: ${summary.names.join(", ")}`;
  }
</script>

{#snippet sortablePlayerHeader(label: string, table: ProgressionPlayerTable, key: ProgressionPlayerSortKey)}
  <th class="px-3 py-2 font-medium" aria-sort={sortAria(table, key)}>
    <button
      type="button"
      class="flex items-center gap-1 font-medium hover:text-neutral-100"
      onclick={() => updatePlayerSort(table, key)}
    >
      <span>{label}</span>
      <IconArrowUp
        class={`size-3 transition-opacity ${
          isActiveSort(table, key) ? "opacity-100" : "opacity-0"
        } ${playerSort(table)?.direction === "desc" ? "rotate-180" : ""}`}
      />
    </button>
  </th>
{/snippet}

<div
  class="mx-auto flex w-full max-w-[180rem] flex-col gap-3 px-6 pt-3 pb-8 transition-opacity"
  class:opacity-60={loading}
  aria-busy={loading || rangeLoading}
>
  <!-- filter options, defaults to first clear of selected raid at any difficulty -->
  <div class="flex flex-wrap items-center gap-2">
    <select
      class="h-9 min-w-64 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      value={selectedGateId}
      aria-label="Raid gate"
      onchange={(event) => updateGate(event.currentTarget.value)}
    >
      {#each gateOptionGroups as group (group.raid)}
        <optgroup label={group.raid}>
          {#each group.gates as gate (gate.id)}
            <option value={gate.id}>{gate.gate}</option>
          {/each}
        </optgroup>
      {/each}
    </select>

    <select
      class="h-9 min-w-40 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      value={selectedDifficulty}
      onchange={(event) => updateDifficulty(event.currentTarget.value)}
    >
      <option value="">Any difficulty</option>
      {#each difficultyMap as difficulty (difficulty)}
        <option value={difficulty}>{difficulty}</option>
      {/each}
    </select>

    <DateRangePicker
      {startDate}
      {endDate}
      onStartDateChange={updateStartDate}
      onEndDateChange={updateEndDate}
      label="Progression date range"
    />

    <button
      type="button"
      class={`h-9 rounded-md px-3 text-sm font-medium transition-colors disabled:cursor-default disabled:opacity-60 ${
        canLoadStatistics || loading
          ? "bg-accent-600 text-white hover:bg-accent-500 disabled:hover:bg-accent-600"
          : "border border-neutral-700 bg-neutral-800 text-neutral-400 hover:bg-neutral-800"
      }`}
      disabled={!canLoadStatistics}
      onclick={loadStatistics}
    >
      {loading ? "Loading" : "Load"}
    </button>

    <QuickTooltip tooltip="Reset filters" class="block">
      <button
        type="button"
        class="flex size-9 items-center justify-center rounded-md border border-neutral-700 bg-neutral-800 text-neutral-300 hover:bg-neutral-700/70 hover:text-neutral-100 disabled:cursor-default disabled:opacity-40 disabled:hover:bg-neutral-800 disabled:hover:text-neutral-300"
        aria-label="Reset filters"
        disabled={!filtersChanged}
        onclick={resetFilters}
      >
        <IconRotateCcw class="size-4" />
      </button>
    </QuickTooltip>
  </div>

  {#if error}
    <div class="rounded-md border border-red-500/40 bg-red-500/10 p-3 text-red-200">{error}</div>
  {/if}

  {#if statistics}
    <!-- summary cards -->
    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-6">
      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">Pulls</div>
        <div class="text-2xl font-semibold">{statistics.summary.attempts}</div>
        <div class="text-xs text-neutral-500">
          {statistics.summary.clears} clear{statistics.summary.clears === 1 ? "" : "s"}, {statistics.summary.wipes} wipes
        </div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">Prog Time</div>
        <div class="text-2xl font-semibold">{formatTotalDuration(statistics.summary.totalDuration)}</div>
        <div class="text-xs text-neutral-500">
          {formatDate(statistics.summary.firstPull)} - {formatDate(statistics.summary.lastPull)}
        </div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">Best Pull</div>
        <div class="text-2xl font-semibold">
          {statistics.summary.clears > 0
            ? "Cleared"
            : progressLabel({
                bestProgressBars: statistics.summary.bestProgressBars,
                bestProgressPercent: statistics.summary.bestProgressPercent
              })}
        </div>
        <div class="text-xs text-neutral-500">clear rate {formatPercent(statistics.summary.clearRate)}</div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">Avg Team DPS</div>
        <div class="text-2xl font-semibold">{formatDps(statistics.summary.averageTeamDps)}</div>
        <div class="text-xs text-neutral-500">
          avg pull duration {formatDuration(statistics.summary.averageDuration)}
        </div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="flex items-center justify-between gap-2">
          <div class="text-xs text-neutral-400">Least Deaths/Pull</div>
        </div>
        <div class="flex min-w-0 items-baseline gap-2">
          <div class="text-2xl leading-tight font-semibold">{formatDeathRate(leastDeathsPerPull)}</div>
          <QuickTooltip tooltip={deathRateTooltip(leastDeathsPerPull)} class="min-w-0">
            <div class="truncate text-sm text-neutral-300">{deathRateNamesLabel(leastDeathsPerPull)}</div>
          </QuickTooltip>
        </div>
        <div class="w-min truncate text-xs text-neutral-500">
          <QuickTooltip tooltip={deathRateTooltip(mostDeathsPerPull)}>
            Most: {formatDeathRate(mostDeathsPerPull)} - {deathRateNamesLabel(mostDeathsPerPull)}
          </QuickTooltip>
        </div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">First Clear</div>
        <div class="text-2xl font-semibold">{formatDuration(statistics.summary.firstClearDuration)}</div>
        <div class="text-xs text-neutral-500">{formatDateTime(statistics.summary.firstClear)}</div>
      </div>
    </div>

    <!--  dps/support player total pulls breakdown  -->

    <div class="grid grid-cols-1 gap-3 xl:grid-cols-2">
      <div class="overflow-hidden rounded-md border border-neutral-700/70 bg-neutral-800/80">
        <!-- dps section -->
        <div class="flex items-center justify-between border-b border-neutral-700/70 px-3 py-2">
          <h2 class="font-medium">DPS</h2>
          <span class="text-xs text-neutral-500">{dpsPlayers.length} players</span>
        </div>
        {#if dpsPlayers.length > 0}
          <div class="max-h-[26rem] overflow-auto">
            <table class="w-full min-w-[60rem] text-left text-xs">
              <thead class="sticky top-0 z-10 bg-neutral-900/95 text-neutral-400">
                <tr>
                  <th class="px-3 py-2 font-medium">Name</th>
                  <th class="px-3 py-2 font-medium">Spec</th>
                  <th class="px-3 py-2 font-medium">Pulls</th>
                  <th class="px-3 py-2 font-medium">Clears</th>
                  {@render sortablePlayerHeader("Avg DPS", "dps", "averageDps")}
                  {@render sortablePlayerHeader("Avg nDPS", "dps", "averageNdps")}
                  {@render sortablePlayerHeader("Avg rDPS", "dps", "averageRdps")}
                  {@render sortablePlayerHeader("Avg Dmg Taken", "dps", "averageDamageTaken")}
                  {@render sortablePlayerHeader("Deaths/Pull", "dps", "deathsPerPull")}
                  {@render sortablePlayerHeader("Deaths", "dps", "totalDeaths")}
                </tr>
              </thead>
              <tbody>
                {#each sortedDpsPlayers as player (player.name)}
                  <tr class="border-t border-neutral-700/70 hover:bg-neutral-700/30">
                    <td class="px-3 py-2">
                      <div class="flex min-w-0 items-center gap-2">
                        {#if player.classId}
                          <img src={getClassIcon(player.classId)} alt="" class="size-5" />
                        {/if}
                        <span class="truncate font-medium">{player.name}</span>
                      </div>
                    </td>
                    <td class="px-3 py-2 text-neutral-300">{player.spec ?? player.class}</td>
                    <td class="px-3 py-2">{player.pulls}</td>
                    <td class="px-3 py-2">{player.clears}</td>
                    <td class="px-3 py-2">{formatDps(player.averageDps)}</td>
                    <td class="px-3 py-2">{formatDps(player.averageNdps)}</td>
                    <td class="px-3 py-2">{formatDps(player.averageRdps)}</td>
                    <td class="px-3 py-2">{formatNumber(player.averageDamageTaken)}</td>
                    <td class="px-3 py-2">{player.deathsPerPull.toFixed(2)}</td>
                    <td class="px-3 py-2">{player.totalDeaths}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="py-12 text-center text-neutral-400">No dps players in this range.</div>
        {/if}
      </div>

      <!-- support section -->

      <div class="overflow-hidden rounded-md border border-neutral-700/70 bg-neutral-800/80">
        <div class="flex items-center justify-between border-b border-neutral-700/70 px-3 py-2">
          <h2 class="font-medium">Supports</h2>
          <span class="text-xs text-neutral-500">{supportPlayers.length} players</span>
        </div>
        {#if supportPlayers.length > 0}
          <div class="max-h-[26rem] overflow-auto">
            <table class={`w-full ${showSupportContribution ? "min-w-[62rem]" : "min-w-[58rem]"} text-left text-xs`}>
              <thead class="sticky top-0 z-10 bg-neutral-900/95 text-neutral-400">
                <tr>
                  <th class="px-3 py-2 font-medium">Name</th>
                  <th class="px-3 py-2 font-medium">Spec</th>
                  <th class="px-3 py-2 font-medium">Pulls</th>
                  <th class="px-3 py-2 font-medium">Clears</th>
                  {#if showSupportContribution}
                    <th class="px-3 py-2 font-medium">Contrib</th>
                  {/if}
                  {@render sortablePlayerHeader("AP", "support", "averageSupportAp")}
                  {@render sortablePlayerHeader("Brand", "support", "averageSupportBrand")}
                  {@render sortablePlayerHeader("Identity", "support", "averageSupportIdentity")}
                  {@render sortablePlayerHeader("T", "support", "averageSupportHyper")}
                  {@render sortablePlayerHeader("Avg Dmg Taken", "support", "averageDamageTaken")}
                  {@render sortablePlayerHeader("Deaths/Pull", "support", "deathsPerPull")}
                  {@render sortablePlayerHeader("Deaths", "support", "totalDeaths")}
                </tr>
              </thead>
              <tbody>
                {#each sortedSupportPlayers as player (player.name)}
                  <tr class="border-t border-neutral-700/70 hover:bg-neutral-700/30">
                    <td class="px-3 py-2">
                      <div class="flex min-w-0 items-center gap-2">
                        {#if player.classId}
                          <img src={getClassIcon(player.classId)} alt="" class="size-5" />
                        {/if}
                        <span class="truncate font-medium">{player.name}</span>
                      </div>
                    </td>
                    <td class="px-3 py-2 text-neutral-300">{player.spec ?? player.class}</td>
                    <td class="px-3 py-2">{player.pulls}</td>
                    <td class="px-3 py-2">{player.clears}</td>
                    {#if showSupportContribution}
                      <td class="px-3 py-2">{formatRatioPercent(player.averageSupportContribution)}</td>
                    {/if}
                    <td class="px-3 py-2">{formatRatioPercent(player.averageSupportAp)}</td>
                    <td class="px-3 py-2">{formatRatioPercent(player.averageSupportBrand)}</td>
                    <td class="px-3 py-2">{formatRatioPercent(player.averageSupportIdentity)}</td>
                    <td class="px-3 py-2">{formatRatioPercent(player.averageSupportHyper)}</td>
                    <td class="px-3 py-2">{formatNumber(player.averageDamageTaken)}</td>
                    <td class="px-3 py-2">{player.deathsPerPull.toFixed(2)}</td>
                    <td class="px-3 py-2">{player.totalDeaths}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="py-12 text-center text-neutral-400">No support players in this range.</div>
        {/if}
      </div>
    </div>

    <!-- full list of pulls -->

    <div class="overflow-hidden rounded-md border border-neutral-700/70 bg-neutral-800/80">
      <div class="flex items-center justify-between border-b border-neutral-700/70 px-3 py-2">
        <h2 class="font-medium">Pull History</h2>
        <span class="text-xs text-neutral-500">{pullRows.length} pulls</span>
      </div>
      {#if pullRows.length > 0}
        <div class="max-h-[22rem] overflow-auto">
          <table class="w-full min-w-[58rem] text-left text-xs">
            <thead class="sticky top-0 z-10 bg-neutral-900/95 text-neutral-400">
              <tr>
                <th class="px-3 py-2 font-medium">ID</th>
                <th class="px-3 py-2 font-medium">Boss</th>
                <th class="px-3 py-2 font-medium">Difficulty</th>
                <th class="px-3 py-2 font-medium">Duration</th>
                <th class="px-3 py-2 font-medium">Result</th>
                <th class="px-3 py-2 font-medium">HP</th>
                <th class="px-3 py-2 font-medium">Team DPS</th>
                <th class="px-3 py-2 font-medium">Deaths</th>
                <th class="px-3 py-2 font-medium">Local Player</th>
              </tr>
            </thead>
            <tbody>
              {#each pullRows as pull (pull.id)}
                <tr class="border-t border-neutral-700/70 hover:bg-neutral-700/30">
                  <td class="px-3 py-2">
                    <a href={resolve(`/logs/${pull.id}`)} class="text-accent-500 hover:text-accent-400">{pull.id}</a>
                  </td>
                  <td class="max-w-56 truncate px-3 py-2" title={pull.bossName}>{pull.bossName}</td>
                  <td class="px-3 py-2">
                    {#if pull.difficulty}
                      {@render difficultyColor(pull.difficulty)}
                    {:else}
                      <span class="text-neutral-500">-</span>
                    {/if}
                  </td>
                  <td class="px-3 py-2">{formatDuration(pull.duration)}</td>
                  <td class="px-3 py-2">
                    <span
                      class={`rounded px-1.5 py-0.5 ${
                        pull.cleared ? "bg-lime-500/15 text-lime-300" : "bg-neutral-700/60 text-neutral-300"
                      }`}
                    >
                      {pull.cleared ? "Clear" : "Wipe"}
                    </span>
                  </td>
                  <td class="px-3 py-2">
                    <div class="flex min-w-28 items-center gap-2">
                      <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-neutral-700">
                        <div
                          class="h-full rounded-full bg-accent-500"
                          style={`width: ${progressCompletePercent(pull.progressPercent, pull.cleared)}%`}
                        ></div>
                      </div>
                      <span class="w-12 text-right">{pullProgressLabel(pull)}</span>
                    </div>
                  </td>
                  <td class="px-3 py-2">{formatDps(pull.teamDps)}</td>
                  <td class="px-3 py-2">{pull.deaths}</td>
                  <td class="px-3 py-2">{pull.localPlayer}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <div class="py-12 text-center text-neutral-400">No pulls in this range.</div>
      {/if}
    </div>
  {:else}
    <div class="rounded-md border border-neutral-700 bg-neutral-800/80 p-3 text-neutral-300">
      {loading ? "Loading progression..." : hasLoaded ? "No progression data." : "No progression loaded."}
    </div>
  {/if}
</div>
