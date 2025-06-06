<script lang="ts">
  import { classNameToClassId } from "$lib/constants/classes";
  import { encounterMap } from "$lib/constants/encounters";
  import { encounterFilter, settings } from "$lib/stores.svelte";
  import { type EncountersOverview } from "$lib/types";
  import { invoke } from "@tauri-apps/api";
  import { untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import Header from "../Header.svelte";
  import EncountersTable from "./EncountersTable.svelte";
  import Pages from "./Pages.svelte";
  import Search from "./Search.svelte";

  let overview: EncountersOverview | null = $state(null);
  let container = $state<HTMLDivElement | null>(null);

  let selectMode = $state(false);
  let selected = $state(new SvelteSet<number>());

  async function loadEncounters() {
    // start or space (^|\s) + word (\w+) + colon or space or end (:|\s|$)
    // using lookbehind (?<=) and lookahead (?=) https://regex101.com/r/1cMFH8/4
    // if word is a valid className, replace it with the classId
    // example: "bard:Anyduck shadowhunter" -> "204:Anyduck 403"
    let searchQuery = encounterFilter.search.replace(/(?<=^|\s)\w+(?=:|\s|$)/g, (word: string) => {
      const className = word[0].toUpperCase() + word.substring(1).toLowerCase();
      return String(classNameToClassId[className] || word);
    });

    let raidBosses = Array.from(encounterFilter.bosses);
    if (encounterFilter.encounters.size > 0) {
      for (const encounter of encounterFilter.encounters) {
        const raid = encounter.substring(0, encounter.lastIndexOf(" "));
        raidBosses.push(...encounterMap[raid][encounter]);
      }
    }

    let overview: EncountersOverview = await invoke("load_encounters_preview", {
      page: encounterFilter.page,
      pageSize: settings.app.general.logsPerPage,
      search: searchQuery,
      filter: {
        minDuration: encounterFilter.minDuration,
        bosses: raidBosses,
        cleared: encounterFilter.cleared,
        favorite: encounterFilter.favorite,
        difficulty: encounterFilter.difficulty,
        sort: encounterFilter.sort,
        order: encounterFilter.order
      }
    });

    return overview;
  }

  let refresh = $state(false);

  $effect.pre(() => {
    refresh;
    (async () => {
      overview = await loadEncounters();
      if (container) {
        container.scrollTop = 0;
      }
    })();
  });

  let once = $state(false);
  // Reset the page to 1 when any filter changes, except for the first load
  $effect(() => {
    encounterFilter.search;
    encounterFilter.minDuration;
    encounterFilter.encounters;
    encounterFilter.bosses;
    encounterFilter.cleared;
    encounterFilter.favorite;
    encounterFilter.difficulty;
    encounterFilter.sort;
    encounterFilter.order;

    // *searching* is true when its not the first load
    const searching = untrack(() => once);
    if (searching) {
      encounterFilter.page = 1;
    }
    once = true;
  });
</script>

<div>
  <Header title="Past Encounters">
    <button
      class="bg-accent-500/70 hover:bg-accent-500/60 rounded-md p-1"
      onclick={() => {
        refresh = !refresh;
        encounterFilter.page = 1;
      }}
    >
      Refresh
    </button>
  </Header>
  <div class="mx-auto flex max-w-[180rem] flex-col justify-between gap-1 px-6 py-1" style="height: calc(100vh - 4rem);">
    <div class="flex flex-col gap-1">
      <Search bind:selectMode bind:selected bind:refresh />
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
    <Pages bind:page={encounterFilter.page} total={overview?.totalEncounters} />
  </div>
</div>
