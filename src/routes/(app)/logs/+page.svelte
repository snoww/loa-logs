<script lang="ts">
  import { classNameToClassId } from "$lib/constants/classes";
  import { encounterMap } from "$lib/constants/encounters";
  import { encounterFilter, settings, type EncounterFilter } from "$lib/stores.svelte";
  import { type EncountersOverview } from "$lib/types";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import Header from "../Header.svelte";
  import EncountersTable from "./EncountersTable.svelte";
  import Pages from "./Pages.svelte";
  import Search from "./Search.svelte";
  import { loadEncountersPreview } from "$lib/api";
  import type { MouseEventHandler } from "svelte/elements";
  import { replaceClassNamesWithIds } from "$lib/utils";

  let overview: EncountersOverview | null = $state(null);
  let container = $state<HTMLDivElement | null>(null);
  let selectMode = $state(false);
  let selected = $state(new SvelteSet<number>());

  onMount(() => {

    encounterFilter.update(filter => {
      filter.minDuration = settings.app.logs.minEncounterDuration;
      return filter
    })

    encounterFilter.subscribe((value: EncounterFilter) => {
      if (container) {
        container.scrollTop = 0;
      }
      loadEncounters(value);
    })
  })

  const onLogsPerPage = (value: number) => {
    settings.app.general.logsPerPage = value;
    encounterFilter.update(filter => filter);
  }

  const onRefresh: MouseEventHandler<HTMLElement> = () => {
    encounterFilter.update(filter => {
      filter.page = 1;
      return filter
    });
  }

  async function loadEncounters(filter: EncounterFilter): Promise<void> {
    
    let searchQuery = replaceClassNamesWithIds(filter.search, classNameToClassId);
    let raidBosses = Array.from(filter.bosses);
    if (filter.encounters.size > 0) {
      for (const encounter of filter.encounters) {
        const raid = encounter.substring(0, encounter.lastIndexOf(" "));
        raidBosses.push(...encounterMap[raid][encounter]);
      }
    }

    const raidType = page.url.searchParams.get("raidType");
    const { showRaidsOnly, logsPerPage } = settings.app.general;
    const criteria = {
      page: filter.page,
      pageSize: logsPerPage,
      search: searchQuery,
      filter: {
        raidType,
        minDuration: filter.minDuration,
        maxDuration: 3600,
        bosses: raidBosses,
        cleared: filter.cleared,
        favorite: filter.favorite,
        difficulty: filter.difficulty,
        sort: filter.sort,
        order: filter.order,
        raidsOnly: showRaidsOnly,
        bossOnlyDamage: false,
      }
    };

    console.log(criteria);
    overview = await loadEncountersPreview(criteria);
  }

</script>

<div>
  <Header title="Past Encounters">
    <button
      class="bg-accent-500/70 hover:bg-accent-500/60 rounded-md p-1"
      onclick={onRefresh}
    >
      Refresh
    </button>
  </Header>
  <div class="mx-auto flex max-w-[180rem] flex-col justify-between gap-1 px-6 py-1" style="height: calc(100vh - 4rem);">
    <div class="flex flex-col gap-1">
      <Search bind:selectMode bind:selected />
      <div
        class="overflow-y-auto overflow-x-hidden rounded-md border border-neutral-700/70"
        style="max-height: calc(100vh - 10.5rem);"
        bind:this={container}
      >
        {#if overview}
          <EncountersTable {overview} {selectMode} bind:selected />
        {/if}
      </div>
      {#if !overview || overview?.encounters.length === 0}
        <p class="p-2">No encounters found.</p>
      {/if}
    </div>
    <Pages bind:logsPerPage={() => settings.app.general.logsPerPage, onLogsPerPage} bind:page={$encounterFilter.page} total={overview?.totalEncounters} />
  </div>
</div>
