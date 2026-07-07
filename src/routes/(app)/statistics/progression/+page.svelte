<script lang="ts">
  import { resolve } from "$app/paths";
  import { getRaidProgressionRange, getRaidProgressionStatistics } from "$lib/api";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { difficultyColor } from "$lib/components/Snippets.svelte";
  import { difficultyMap, encounterMap } from "$lib/constants/encounters";
  import { IconRotateCcw } from "$lib/icons";
  import type { RaidProgressionPlayer, RaidProgressionPull, RaidProgressionStatistics } from "$lib/types";
  import { getClassIcon } from "$lib/utils";
  import { onMount } from "svelte";

  import {
    dateToEndTime,
    dateToStartTime,
    formatDateTime,
    formatDecimal,
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
    error: string;
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
  let loading = $state(false);
  let rangeLoading = $state(false);
  let hasLoaded = $state(false);
  let error = $state("");
  let initialized = $state(false);
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
      endDate !== defaultEndDate ||
      statistics !== null ||
      hasLoaded
  );
  let dpsPlayers = $derived((statistics?.players ?? []).filter((player) => !player.isSupport));
  let supportPlayers = $derived((statistics?.players ?? []).filter((player) => player.isSupport));
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
        hasLoaded = true;
      }
    } catch (err) {
      console.error(err);
      if (currentRequest === requestId) {
        statistics = null;
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

  function clearLoadedStatistics() {
    ++requestId;
    loading = false;
    statistics = null;
    hasLoaded = false;
  }

  function updateGate(value: string) {
    if (selectedGateId === value) return;
    selectedGateId = value;
    selectedDifficulty = "";
    clearLoadedStatistics();
    loadDefaultRange();
  }

  function updateDifficulty(value: string) {
    if (selectedDifficulty === value) return;
    selectedDifficulty = value;
    clearLoadedStatistics();
    loadDefaultRange();
  }

  function updateStartDate(value: string) {
    ++rangeRequestId;
    rangeLoading = false;
    clearLoadedStatistics();
    startDate = value;
  }

  function updateEndDate(value: string) {
    ++rangeRequestId;
    rangeLoading = false;
    clearLoadedStatistics();
    endDate = value;
  }

  function resetFilters() {
    clearLoadedStatistics();
    selectedGateId = defaultGateId;
    selectedDifficulty = "";
    startDate = "";
    endDate = "";
    loadDefaultRange();
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
</script>

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

    <input
      type="date"
      class="h-9 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      value={startDate}
      onchange={(event) => updateStartDate(event.currentTarget.value)}
      aria-label="Start date"
    />

    <input
      type="date"
      class="h-9 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      value={endDate}
      onchange={(event) => updateEndDate(event.currentTarget.value)}
      aria-label="End date"
    />

    <button
      type="button"
      class="h-9 rounded-md bg-accent-600 px-3 text-sm font-medium text-white hover:bg-accent-500 disabled:cursor-default disabled:opacity-50 disabled:hover:bg-accent-600"
      disabled={loading || rangeLoading || !selectedGateOption}
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
  {:else if statistics}
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
        <div class="text-xs text-neutral-400">Avg Damage Taken</div>
        <div class="text-2xl font-semibold">{formatNumber(statistics.summary.averageDamageTaken)}</div>
        <div class="text-xs text-neutral-500">{formatDecimal(statistics.summary.averageDeaths)} deaths per pull</div>
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
                  <th class="px-3 py-2 font-medium">Avg DPS</th>
                  <th class="px-3 py-2 font-medium">Avg nDPS</th>
                  <th class="px-3 py-2 font-medium">Avg rDPS</th>
                  <th class="px-3 py-2 font-medium">Avg Dmg Taken</th>
                  <th class="px-3 py-2 font-medium">Deaths/Pull</th>
                  <th class="px-3 py-2 font-medium">Deaths</th>
                </tr>
              </thead>
              <tbody>
                {#each dpsPlayers as player (player.name)}
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
            <table class="w-full min-w-[58rem] text-left text-xs">
              <thead class="sticky top-0 z-10 bg-neutral-900/95 text-neutral-400">
                <tr>
                  <th class="px-3 py-2 font-medium">Name</th>
                  <th class="px-3 py-2 font-medium">Spec</th>
                  <th class="px-3 py-2 font-medium">Pulls</th>
                  <th class="px-3 py-2 font-medium">Clears</th>
                  <th class="px-3 py-2 font-medium">AP</th>
                  <th class="px-3 py-2 font-medium">Brand</th>
                  <th class="px-3 py-2 font-medium">Identity</th>
                  <th class="px-3 py-2 font-medium">T</th>
                  <th class="px-3 py-2 font-medium">Avg Dmg Taken</th>
                  <th class="px-3 py-2 font-medium">Deaths/Pull</th>
                  <th class="px-3 py-2 font-medium">Deaths</th>
                </tr>
              </thead>
              <tbody>
                {#each supportPlayers as player (player.name)}
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
