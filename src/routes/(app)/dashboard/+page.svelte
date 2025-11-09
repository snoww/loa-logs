<script lang="ts">
  import { getRaidStats } from "$lib/api";
  import Header from "../Header.svelte";
  import { onMount } from "svelte";
  import { getBossImage, getDateRange } from "./utils";
  import IconBlocksWave from "~icons/svg-spinners/blocks-wave";
  import IconArrowLeft from "~icons/lucide/arrow-left";
  import IconArrowRight from "~icons/lucide/arrow-right";
  import IconDoubleArrowRight from "~icons/tabler/chevrons-right";
  import IconActivity from "~icons/tabler/activity";
  import IconSparkles from "~icons/tabler/sparkles";
  import IconExternalLink from "~icons/lucide/external-link";
  import type { RaidStats } from "$lib/stats";
  import type { ChangeEventHandler, MouseEventHandler } from "svelte/elements";

  type PageState = { isLoading: true } | {
    isLoading: false;
    results: RaidStats[];
    filtered: RaidStats[];
  }

  let criteria = $state({
    date: new Date(),
    weeks: 0,
    formatted: getDateRange(new Date()).formatted,
    includeAllGates: false,
    includeGuardianRaids: false
  });
  let pageState = $state<PageState>({
    isLoading: true,
  });

  onMount(() => {
    onLoad()
  });

  async function onLoad() {
    const { dateFrom, dateTo, formatted } = getDateRange(criteria.date);

    const args = {
      dateFrom,
      dateTo,
    }

    const results = await getRaidStats(args);
    let filtered: RaidStats[] = filterAndSort(results.items);

    criteria.formatted = formatted;
    pageState = {
      isLoading: false,
      results: results.items,
      filtered
    }
  }

  const onToggle: ChangeEventHandler<HTMLInputElement> = (event) => {
    const field = event.currentTarget.dataset.field as keyof typeof criteria;

    if(pageState.isLoading) {
      return
    }
    
    if(field !== "includeAllGates" && field !== "includeGuardianRaids"){
      return;
    }

    criteria[field] = !criteria[field];
    let filtered: RaidStats[] = filterAndSort(pageState.results);

    pageState = {
      ...pageState,
      filtered
    }
  }

  function filterAndSort(items: RaidStats[]): RaidStats[] {
    return items
      .filter(pr => {
        if (!criteria.includeAllGates && !pr.isFinalGate) return false
        if (!criteria.includeGuardianRaids && pr.isGuardianRaid) return false
        return true
      })
      .toSorted((a,b) => a.order - b.order)
  }

  const onPivot: MouseEventHandler<HTMLElement> = (event) => {
    const weeks = Number(event.currentTarget.dataset.value);

    if (weeks === 0) {
      criteria.date = new Date();
      criteria.weeks = weeks;
    } else {
      criteria.date = new Date(criteria.date.setDate(criteria.date.getDate() + 7 * weeks));
      criteria.weeks += weeks;
    }

    onLoad();
  }

</script>

<Header title="My raids" />
<div class="flex-1">
  <div class="flex sticky items-center gap-3 top-16 bg-neutral-900/40 backdrop-blur z-30 p-3">
    <span class="font-medium">{criteria.formatted}</span>
    <button data-value="-1" type="button" onclick={onPivot} class="ml-auto"><IconArrowLeft/></button>
    <button data-value="+1" type="button" onclick={onPivot} disabled={criteria.weeks > -1} class="disabled:opacity-30 disabled:pointer-events-non"><IconArrowRight/></button>
    <button data-value="0" type="button" onclick={onPivot} disabled={criteria.weeks > -1} class=" disabled:opacity-30 disabled:pointer-events-non"><IconDoubleArrowRight class="w-5 h-5"/></button>
    <label for="all-gates">
      <input
        data-field="includeAllGates"
        id="all-gates"
        type="checkbox"
        checked={criteria.includeAllGates}
        onchange={onToggle}
        class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0">
      All gates
    </label>
    <label for="guardian-raids">
      <input
        data-field="includeGuardianRaids"
        id="guardian-raids"
        type="checkbox"
        checked={criteria.includeGuardianRaids}
        onchange={onToggle}
        class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0">
      Guardian raids
    </label>
  </div>
  {#if pageState.isLoading }
    <div class="w-screen h-full flex items-center justify-center">
      <IconBlocksWave class="w-10 h-10"/>
    </div>
  {:else}
    <div class="grid grid-cols-3">
      {#each pageState.filtered as item, index (item.raidType)}
        <div
              class={`relative rounded-lg overflow-hidden
              ${pageState.filtered.length === 5 && index >= 3 ? "col-span-1" : ""}`}
          >
              <img
                  class="w-full h-full object-cover"
                  src={getBossImage(item.raidType)}
                  alt={item.raidType}
              />
              <div class="absolute top-1 right-1 text-gray-400 text-sm font-bold px-2 py-1 rounded-full z-10">
                  {item.count} {item.count === 1 ? " run" : " runs"}
              </div>
              <div class="absolute inset-0 pointer-events-none bg-[radial-gradient(ellipse_at_center,_rgba(0,0,0,0.4)_0%,_rgba(0,0,0,0.6)_100%)]"></div>
              <div class="absolute inset-0 flex justify-center items-center">
                  <div class="relative px-2 py-2 text-gray-100 text-sm font-medium text-center">
                      <div class="font-bold text-base flex items-center justify-center gap-2">
                        {item.name}
                        <a href={`/logs?raidType=${item.raidType}`}><IconExternalLink/></a>
                      </div>
                      {#if item.dps}
                          <div class="flex items-center justify-center gap-1 mt-1">
                              <IconActivity /> {item.dps.formatted}
                          </div>
                      {/if}
                      {#if item.uptimes}
                          <div class="flex items-center justify-center gap-1 text-xs mt-1">
                              <IconSparkles />
                              {item.uptimes[0]} %
                              {item.uptimes[1]} %
                              {item.uptimes[2]} %
                              {item.uptimes[3]} %
                          </div>
                      {/if}
                  </div>
              </div>
          </div>
      {/each}
    </div>
  {/if}
</div>