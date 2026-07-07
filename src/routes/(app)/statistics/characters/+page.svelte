<script lang="ts">
  import { resolve } from "$app/paths";
  import {
    getCharacterStatistics,
    getLocalCharacters,
    type CharacterInfo,
    type CharacterStatisticsCriteria
  } from "$lib/api";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { difficultyColor } from "$lib/components/Snippets.svelte";
  import { difficultyMap, encounterMap, raidGates } from "$lib/constants/encounters";
  import { IconRotateCcw } from "$lib/icons";
  import type { CharacterStatistics, RaidStatisticsRow, RecentBestEncounter } from "$lib/types";
  import { getClassIcon, isSupportClassId, isSupportSpec, SUPPORT_SPECS } from "$lib/utils";
  import { onMount } from "svelte";

  import {
    formatDps,
    formatDuration,
    formatLongDate as formatDate,
    formatPercent,
    formatRatioPercent
  } from "../format";

  type Range = CharacterStatisticsCriteria["range"];
  type Mode = CharacterStatisticsCriteria["mode"];
  type DamageType = NonNullable<CharacterStatisticsCriteria["damageType"]>;

  const ranges: { value: Range; label: string }[] = [
    { value: "current_week", label: "Current week" },
    { value: "previous_week", label: "Previous week" },
    { value: "last4_weeks", label: "Last 4 weeks" },
    { value: "last8_weeks", label: "Last 8 weeks" },
    { value: "all", label: "All time" }
  ];

  const damageTypes: { value: DamageType; label: string; tooltip: string }[] = [
    { value: "dps", label: "DPS", tooltip: "" },
    { value: "ndps", label: "nDPS", tooltip: "" },
    { value: "rdps", label: "rDPS", tooltip: "" }
  ];

  const raidBosses = Object.values(encounterMap)
    .flatMap((raid) => Object.values(raid).flat())
    .filter((boss, index, bosses) => bosses.indexOf(boss) === index);

  let characters = $state<CharacterInfo[]>([]);
  let selectedCharacter = $state("");
  let selectedRange = $state<Range>("current_week");
  let selectedMode = $state<Mode>("damage");
  let selectedDamageType = $state<DamageType>("dps");
  let selectedRaid = $state("");
  let selectedDifficulty = $state("");
  let statistics = $state<CharacterStatistics | null>(null);
  let loading = $state(false);
  let error = $state("");
  let initialized = $state(false);
  let requestId = 0;

  let selectedBosses = $derived(
    selectedRaid
      ? Object.values(encounterMap[selectedRaid])
          .flat()
          .filter((boss, index, bosses) => bosses.indexOf(boss) === index)
      : []
  );

  let includedBosses = $derived(selectedRaid ? selectedBosses : raidBosses);
  let selectedCharacterInfo = $derived(characters.find((entry) => entry.name === selectedCharacter));
  let canSelectSupportMode = $derived(isSupportClassId(selectedCharacterInfo?.classId));
  let isSupportMode = $derived(selectedMode === "support");
  let selectedDamageTypeInfo = $derived(
    damageTypes.find((damageType) => damageType.value === selectedDamageType) ?? damageTypes[0]
  );
  let filtersChanged = $derived(
    selectedMode !== defaultModeForCharacter(selectedCharacterInfo) ||
      selectedDamageType !== "dps" ||
      selectedRange !== "current_week" ||
      selectedRaid !== "" ||
      selectedDifficulty !== ""
  );
  let raidRows = $derived<RaidStatisticsRow[]>(statistics?.raids ?? []);
  let raidLogCount = $derived(statistics?.raids.reduce((total, row) => total + row.attempts, 0) ?? 0);
  let recentBestRows = $derived<RecentBestEncounter[]>(
    [...(statistics?.recentBests ?? [])]
      .sort((a, b) => {
        const aValue = isSupportMode ? (a.supportContribution ?? 0) : (metricValue(a, "recent") ?? 0);
        const bValue = isSupportMode ? (b.supportContribution ?? 0) : (metricValue(b, "recent") ?? 0);
        return bValue - aValue;
      })
      .slice(0, 5)
  );

  onMount(async () => {
    characters = await getLocalCharacters();
    applyUrlState();
    initialized = true;
  });

  $effect(() => {
    if (!initialized) return;

    if (!canSelectSupportMode && selectedMode === "support") {
      selectedMode = "damage";
      return;
    }

    selectedCharacter;
    selectedRange;
    selectedMode;
    selectedRaid;
    selectedDifficulty;

    if (!selectedCharacter) {
      statistics = null;
      return;
    }

    loadStatistics();
  });

  $effect(() => {
    if (!initialized) return;

    selectedCharacter;
    selectedRange;
    selectedMode;
    selectedDamageType;
    selectedRaid;
    selectedDifficulty;

    updateUrlState();
  });

  async function loadStatistics() {
    const requestCharacter = selectedCharacter;
    const currentRequest = ++requestId;
    loading = true;
    error = "";

    try {
      const result = await getCharacterStatistics({
        character: requestCharacter,
        range: selectedRange,
        mode: selectedMode,
        damageType: "dps",
        bossToRaid: raidGates,
        bosses: includedBosses,
        includedSpecs: canSelectSupportMode && selectedMode === "support" ? SUPPORT_SPECS : [],
        excludedSpecs: canSelectSupportMode && selectedMode === "damage" ? SUPPORT_SPECS : [],
        difficulty: selectedDifficulty,
        minDuration: 10
      });

      if (requestCharacter === selectedCharacter && currentRequest === requestId) {
        statistics = result;
      }
    } catch (err) {
      console.error(err);
      error = "Could not load statistics.";
    } finally {
      if (requestCharacter === selectedCharacter && currentRequest === requestId) {
        loading = false;
      }
    }
  }

  function gateLabel(row: RaidStatisticsRow) {
    return raidGates[row.bossName] ?? row.bossName;
  }

  function metricValue(
    source: Partial<CharacterStatistics["summary"] & RaidStatisticsRow & RecentBestEncounter>,
    metric: "median" | "best" | "p75" | "recent" = "median"
  ) {
    if (metric === "recent") {
      const recent = source as Pick<RecentBestEncounter, "myDps" | "myNdps" | "myRdps">;
      if (selectedDamageType === "rdps") return recent.myRdps;
      if (selectedDamageType === "ndps") return recent.myNdps;
      return recent.myDps;
    }

    const keyed = source as Record<string, number | undefined>;
    const prefix = metric === "best" ? "best" : metric === "p75" ? "p75" : "median";
    if (selectedDamageType === "rdps") return keyed[`${prefix}Rdps`];
    if (selectedDamageType === "ndps") return keyed[`${prefix}Ndps`];
    return keyed[`${prefix}Dps`];
  }

  function applyUrlState() {
    const params = new URLSearchParams(window.location.search);
    const requestedCharacter = params.get("character");
    const character = characters.find((entry) => entry.name === requestedCharacter) ?? characters[0];
    selectedCharacter = character?.name ?? "";

    const requestedRange = params.get("range") as Range | null;
    selectedRange = ranges.some((range) => range.value === requestedRange) ? requestedRange! : "current_week";

    const requestedDamageType = params.get("damageType") as DamageType | null;
    selectedDamageType = damageTypes.some((damageType) => damageType.value === requestedDamageType)
      ? requestedDamageType!
      : "dps";

    const requestedRaid = params.get("raid");
    selectedRaid = requestedRaid && encounterMap[requestedRaid] ? requestedRaid : "";

    const requestedDifficulty = params.get("difficulty");
    selectedDifficulty = requestedDifficulty && difficultyMap.includes(requestedDifficulty) ? requestedDifficulty : "";

    const requestedMode = params.get("mode") as Mode | null;
    const defaultMode = defaultModeForCharacter(character);
    selectedMode =
      requestedMode === "support" && isSupportClassId(character?.classId)
        ? "support"
        : requestedMode === "damage"
          ? "damage"
          : defaultMode;
  }

  function updateUrlState() {
    const params = new URLSearchParams();
    if (selectedCharacter) params.set("character", selectedCharacter);
    if (selectedRange !== "current_week") params.set("range", selectedRange);
    if (selectedMode !== defaultModeForCharacter(selectedCharacterInfo)) params.set("mode", selectedMode);
    if (selectedDamageType !== "dps") params.set("damageType", selectedDamageType);
    if (selectedRaid) params.set("raid", selectedRaid);
    if (selectedDifficulty) params.set("difficulty", selectedDifficulty);

    const query = params.toString();
    const nextUrl = `${window.location.pathname}${query ? `?${query}` : ""}`;
    if (nextUrl !== `${window.location.pathname}${window.location.search}`) {
      window.history.replaceState(window.history.state, "", nextUrl);
    }
  }

  function defaultModeForCharacter(character?: CharacterInfo): Mode {
    return isSupportClassId(character?.classId) && isSupportSpec(character?.spec) ? "support" : "damage";
  }

  function selectCharacter(character: string) {
    selectedCharacter = character;
    selectedMode = defaultModeForCharacter(characters.find((entry) => entry.name === character));
  }

  function resetFilters() {
    selectedMode = defaultModeForCharacter(selectedCharacterInfo);
    selectedDamageType = "dps";
    selectedRange = "current_week";
    selectedRaid = "";
    selectedDifficulty = "";
  }
</script>

<div
  class="mx-auto flex h-full min-h-0 max-w-[180rem] flex-col gap-3 overflow-y-auto px-6 py-3 transition-opacity"
  class:opacity-60={loading}
  aria-busy={loading}
>
  <!--  filter options  -->
  <div class="flex flex-wrap items-center gap-2">
    <select
      class="h-9 min-w-72 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      value={selectedCharacter}
      onchange={(event) => selectCharacter(event.currentTarget.value)}
      disabled={characters.length === 0}
    >
      {#each characters as character (character.name)}
        <option value={character.name}>
          {character.name}{character.maxGearScore > 0 ? ` - ${Math.round(character.maxGearScore)}` : ""}
        </option>
      {/each}
    </select>

    <!--  support classes can select dps spec  -->
    {#if canSelectSupportMode}
      <select
        class="h-9 min-w-32 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
        bind:value={selectedMode}
      >
        <option value="damage">Damage</option>
        <option value="support">Support</option>
      </select>
    {/if}

    {#if !isSupportMode}
      <QuickTooltip tooltip={selectedDamageTypeInfo.tooltip} class="block">
        <select
          class="h-9 min-w-28 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
          bind:value={selectedDamageType}
        >
          {#each damageTypes as damageType (damageType.value)}
            <option value={damageType.value}>{damageType.label}</option>
          {/each}
        </select>
      </QuickTooltip>
    {/if}

    <select
      class="h-9 min-w-44 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      bind:value={selectedRange}
    >
      {#each ranges as range (range.value)}
        <option value={range.value}>{range.label}</option>
      {/each}
    </select>

    <select
      class="h-9 min-w-56 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      bind:value={selectedRaid}
    >
      <option value="">All raids</option>
      {#each Object.keys(encounterMap).reverse() as raid (raid)}
        <option value={raid}>{raid}</option>
      {/each}
    </select>

    <select
      class="h-9 min-w-40 rounded-md border border-neutral-700 bg-neutral-800 px-2 text-sm text-neutral-200 focus:border-accent-500 focus:ring-0"
      bind:value={selectedDifficulty}
    >
      <option value="">Any difficulty</option>
      {#each difficultyMap as difficulty (difficulty)}
        <option value={difficulty}>{difficulty}</option>
      {/each}
    </select>

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
  {:else if characters.length === 0 && !loading}
    <div class="rounded-md border border-neutral-700 bg-neutral-800/80 p-3 text-neutral-300">No characters found.</div>
  {:else if statistics}
    <!-- summary stats -->

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-4">
      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="flex h-full items-center gap-3 text-center">
          {#if statistics.character.classId}
            <img src={getClassIcon(statistics.character.classId)} alt="" class="size-10" />
          {/if}
          <div class="min-w-0">
            <div class="truncate text-base font-medium">{statistics.character.name}</div>
            <div class="text-left text-xs text-neutral-400">
              {statistics.character.maxGearScore > 0
                ? Math.round(statistics.character.maxGearScore)
                : "Local character"}
            </div>
          </div>
        </div>
      </div>

      <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
        <div class="text-xs text-neutral-400">Clears</div>
        <div class="text-2xl font-semibold">{statistics.summary.clears}/{statistics.summary.attempts}</div>
        <div class="text-xs text-neutral-500">
          {formatPercent(statistics.summary.clearRate)} clear rate, {statistics.summary.wipes} wipes
        </div>
      </div>

      {#if isSupportMode}
        <QuickTooltip tooltip="Median rDPS contribution from cleared support logs" class="block">
          <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
            <div class="text-xs text-neutral-400">Median Contribution</div>
            <div class="text-2xl font-semibold">
              {formatRatioPercent(statistics.summary.support?.medianContribution)}
            </div>
            <div class="text-xs text-neutral-500">
              best {formatRatioPercent(statistics.summary.support?.bestContribution)}
            </div>
          </div>
        </QuickTooltip>

        <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
          <div class="text-xs text-neutral-400">Buff Uptimes</div>
          <div class="mt-2 grid grid-cols-3 gap-2">
            <div>
              <div class="text-xl leading-6 font-semibold">
                {formatRatioPercent(statistics.summary.support?.ap)}
              </div>
              <div class="text-xs text-neutral-500">AP</div>
            </div>
            <div>
              <div class="text-xl leading-6 font-semibold">
                {formatRatioPercent(statistics.summary.support?.brand)}
              </div>
              <div class="text-xs text-neutral-500">Brand</div>
            </div>
            <div>
              <div class="text-xl leading-6 font-semibold">
                {formatRatioPercent(statistics.summary.support?.identity)}
              </div>
              <div class="text-xs text-neutral-500">Identity</div>
            </div>
          </div>
        </div>
      {:else}
        <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
          <div class="text-xs text-neutral-400">Median {selectedDamageTypeInfo.label}</div>
          <div class="text-2xl font-semibold">{formatDps(metricValue(statistics.summary))}</div>
          <div class="text-xs text-neutral-500">
            cleared pulls, p75 {formatDps(metricValue(statistics.summary, "p75"))}
          </div>
        </div>

        <div class="h-24 rounded-md border border-neutral-700/70 bg-neutral-800/80 p-3">
          <div class="text-xs text-neutral-400">Best {selectedDamageTypeInfo.label}</div>
          <div class="text-2xl font-semibold">{formatDps(metricValue(statistics.summary, "best"))}</div>
          <div class="text-xs text-neutral-500">
            median duration {formatDuration(statistics.summary.medianDuration)}
          </div>
        </div>
      {/if}
    </div>

    <!-- raid stats -->

    <div class="grid grid-cols-1 gap-3 xl:grid-cols-3">
      <div class="overflow-hidden rounded-md border border-neutral-700/70 bg-neutral-800/80 xl:col-span-2">
        <div class="flex items-center justify-between border-b border-neutral-700/70 px-3 py-2">
          <h2 class="font-medium">Raid Breakdown</h2>
          <span class="text-xs text-neutral-500">{raidLogCount} logs</span>
        </div>
        {#if raidRows.length > 0}
          <div class="max-h-[28rem] overflow-auto">
            <table class="w-full min-w-[64rem] text-left text-xs">
              <thead class="sticky top-0 z-10 bg-neutral-900/95 text-neutral-400">
                <tr>
                  <th class="px-3 py-2 font-medium">Raid</th>
                  <th class="px-3 py-2 font-medium">Difficulty</th>
                  <th class="px-3 py-2 font-medium">Clears</th>
                  <th class="px-3 py-2 font-medium">Rate</th>
                  {#if isSupportMode}
                    <th class="px-3 py-2 font-medium">Contribution</th>
                    <th class="px-3 py-2 font-medium">AP</th>
                    <th class="px-3 py-2 font-medium">Brand</th>
                    <th class="px-3 py-2 font-medium">Identity</th>
                    <th class="px-3 py-2 font-medium">T</th>
                  {:else}
                    <th class="px-3 py-2 font-medium">Median {selectedDamageTypeInfo.label}</th>
                    <th class="px-3 py-2 font-medium">Best {selectedDamageTypeInfo.label}</th>
                  {/if}
                  <th class="px-3 py-2 font-medium">Duration</th>
                  <th class="px-3 py-2 font-medium">Last Clear</th>
                </tr>
              </thead>
              <tbody>
                {#each raidRows as row (`${row.bossName}-${row.difficulty ?? ""}`)}
                  <tr class="border-t border-neutral-700/70 hover:bg-neutral-700/30">
                    <td class="px-3 py-2">{gateLabel(row)}</td>
                    <td class="px-3 py-2">
                      {#if row.difficulty}
                        {@render difficultyColor(row.difficulty)}
                      {:else}
                        <span class="text-neutral-500">-</span>
                      {/if}
                    </td>
                    <td class="px-3 py-2">{row.clears}/{row.attempts}</td>
                    <td class="px-3 py-2">{formatPercent(row.clearRate)}</td>
                    {#if isSupportMode}
                      <td class="px-3 py-2">
                        <QuickTooltip tooltip="Median rDPS contribution on cleared pulls" class="w-fit">
                          {formatRatioPercent(row.support?.medianContribution)}
                        </QuickTooltip>
                      </td>
                      <td class="px-3 py-2">
                        <QuickTooltip tooltip="Average attack power buff uptime on cleared pulls" class="w-fit">
                          {formatRatioPercent(row.support?.ap)}
                        </QuickTooltip>
                      </td>
                      <td class="px-3 py-2">
                        <QuickTooltip tooltip="Average brand uptime on cleared pulls" class="w-fit">
                          {formatRatioPercent(row.support?.brand)}
                        </QuickTooltip>
                      </td>
                      <td class="px-3 py-2">
                        <QuickTooltip tooltip="Average identity buff uptime on cleared pulls" class="w-fit">
                          {formatRatioPercent(row.support?.identity)}
                        </QuickTooltip>
                      </td>
                      <td class="px-3 py-2">
                        <QuickTooltip tooltip="Average T skill buff uptime on cleared pulls" class="w-fit">
                          {formatRatioPercent(row.support?.hyper)}
                        </QuickTooltip>
                      </td>
                    {:else}
                      <td class="px-3 py-2">{formatDps(metricValue(row))}</td>
                      <td class="px-3 py-2">{formatDps(metricValue(row, "best"))}</td>
                    {/if}
                    <td class="px-3 py-2">{formatDuration(row.medianDuration)}</td>
                    <td class="px-3 py-2">{formatDate(row.lastClear)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="py-12 text-center text-neutral-400">No raid data in this range.</div>
        {/if}
      </div>

      <!-- list of recent encounters -->

      <div class="grid content-start gap-3">
        <div class="overflow-hidden rounded-md border border-neutral-700/70 bg-neutral-800/80">
          <div class="flex items-center justify-between border-b border-neutral-700/70 px-3 py-2">
            <h2 class="font-medium">Recent Bests</h2>
            <span class="text-xs text-neutral-500">
              {isSupportMode ? "support logs" : selectedDamageTypeInfo.label}
            </span>
          </div>
          {#if recentBestRows.length > 0}
            <div class="divide-y divide-neutral-700/70">
              {#each recentBestRows as best (best.id)}
                <a href={resolve(`/logs/${best.id}`)} class="block px-3 py-2 hover:bg-neutral-700/30">
                  <div class="flex items-center justify-between gap-2">
                    <span class="truncate">{raidGates[best.bossName] ?? best.bossName}</span>
                    <QuickTooltip
                      tooltip={isSupportMode ? "rDPS contribution" : selectedDamageTypeInfo.label}
                      class="shrink-0 font-medium"
                    >
                      {isSupportMode
                        ? formatRatioPercent(best.supportContribution)
                        : formatDps(metricValue(best, "recent"))}
                    </QuickTooltip>
                  </div>
                  <div class="mt-0.5 flex items-center justify-between gap-2 text-xs text-neutral-500">
                    <span>
                      {#if best.difficulty}
                        {@render difficultyColor(best.difficulty)}
                      {:else}
                        -
                      {/if}
                    </span>
                    <span>{formatDate(best.fightStart)} &middot; {formatDuration(best.duration)}</span>
                  </div>
                </a>
              {/each}
            </div>
          {:else}
            <div class="py-12 text-center text-neutral-400">No cleared pulls in this range.</div>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <div class="rounded-md border border-neutral-700 bg-neutral-800/80 p-3 text-neutral-300">
      {loading ? "Loading statistics..." : "Select a character."}
    </div>
  {/if}
</div>
