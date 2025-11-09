<script lang="ts">
  import { deleteEncounters } from "$lib/api";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { classList } from "$lib/constants/classes";
  import { bossList, difficultyMap, encounterMap } from "$lib/constants/encounters";
  import { IconFilter, IconTrash, IconX } from "$lib/icons";
  import { encounterFilter, settings } from "$lib/stores.svelte";
  import { createDialog, createDropdownMenu, melt } from "@melt-ui/svelte";
  import type { FormEventHandler, MouseEventHandler } from "svelte/elements";
  import { SvelteSet } from "svelte/reactivity";
  import { fade, fly } from "svelte/transition";
  import { page } from "$app/state";

  interface Props {
    selectMode: boolean;
    selected: SvelteSet<number>;
  }

  let {
    selectMode = $bindable(),
    selected = $bindable()
  }: Props = $props();

  const {
    elements: { menu, trigger },
    states: { open }
  } = createDropdownMenu({
    escapeBehavior: "close",
    preventScroll: false,
    positioning: {
      placement: "bottom-end",
      gutter: 16
    }
  });

  const {
    elements: { trigger: dialogTrigger, portalled, overlay, content, title, description, close },
    states: { open: dialogOpen }
  } = createDialog();

  let currentTab = $state("Encounters");
  let search = $state($encounterFilter.search || "");
  let active = $derived(
    $encounterFilter.encounters.size > 0 ||
      $encounterFilter.bosses.size > 0 ||
      $encounterFilter.cleared ||
      $encounterFilter.favorite ||
      $encounterFilter.difficulty !== "" ||
      search.length >= 1
  );

  function debounce(fn: FormEventHandler<HTMLInputElement>, milliseconds: number) {
    let timer: number | undefined;

    return (evt: Event & { currentTarget: EventTarget & HTMLInputElement }) => {
      clearTimeout(timer);
      const timeout = search.length >= 1 ? milliseconds : 0;
      timer = setTimeout(() => fn(evt), timeout);
      // currentTarget is null because the event expires
    };
  }

  const clearSearchQuery = () => {
    page.url.searchParams.delete("raidType");
  }

  const handleSearchInput = debounce(() => {
    $encounterFilter.search = search.length >= 1 ? search : "";
  }, 300);

  const onDelete: MouseEventHandler<HTMLElement> = async (event) => {
    await deleteEncounters({
      type: "ids",
      ids: [...selected]
    });
    selected.clear();
    selectMode = false;
    encounterFilter.update(filter => filter);
  }

  const onClearSelected: MouseEventHandler<HTMLElement> = (event) => {
    selectMode = !selectMode;
    if (!selectMode) {
      selected.clear();
    }
  }

  const onClear: MouseEventHandler<HTMLElement> = (event) => {
    clearSearchQuery();
    search = "";
    encounterFilter.update(filter => {
      filter.search = "";
      filter.page = 1;
      filter.bosses = new SvelteSet();
      filter.encounters = new SvelteSet();
      filter.favorite = false;
      filter.cleared = false;
      filter.difficulty = "";
      filter.sort = "id";
      filter.order = "desc";
      return filter;
    })
  }

  const onDifficultySelect: MouseEventHandler<HTMLElement> = (event) => {
    clearSearchQuery();
    const difficulty = event.currentTarget.dataset.difficulty!;
    encounterFilter.update((filter) => {
      filter.page = 1;
      filter.difficulty = filter.difficulty === difficulty ? "" : difficulty;
      return filter;
    })
  }

  const onEncounterSelect: MouseEventHandler<HTMLElement> = (event) => {
    clearSearchQuery();
    const encounter = event.currentTarget.dataset.encounter!;
    encounterFilter.update((filter) => {
      filter.page = 1;
      filter.encounters.has(encounter)
        ? filter.encounters.delete(encounter)
        : filter.encounters.add(encounter);
      return filter
    })
  }

  const onBossSelect: MouseEventHandler<HTMLElement> = (event) => {
    clearSearchQuery();
    const boss = event.currentTarget.dataset.boss!;
    encounterFilter.update((filter) => {
      filter.page = 1;
      filter.bosses.has(boss) ? filter.bosses.delete(boss) : filter.bosses.add(boss);
      return filter
    })
  }

  const onClassSelect: MouseEventHandler<HTMLElement> = (event) => {
    clearSearchQuery();
    const className = event.currentTarget.dataset.classname!;
    search += ` ${className.toLowerCase()}:`;
    encounterFilter.update((filter) => {
      filter.page = 1;
      filter.search = search;
      return filter
    })
  }

  const onTab: MouseEventHandler<HTMLElement> = (event) => {
    const tab = event.currentTarget.dataset.tab!;
    currentTab = tab;
  }

</script>

{#snippet tab(tab: string)}
  <button
    type="button"
    data-tab={tab}
    class="hover:text-accent-500 p-2 first:rounded-tl {currentTab === tab ? 'bg-neutral-800' : ''}"
    onclick={onTab}
  >
    {tab}
  </button>
{/snippet}

<div class="flex items-center justify-between py-1">
  <div class="relative flex items-center gap-2">
    <button class="hover:text-accent-500 absolute left-2.5" use:melt={$trigger}>
      <QuickTooltip tooltip="Filter encounters" placement="top">
        <IconFilter class={active ? "text-accent-500" : ""} />
      </QuickTooltip>
    </button>
    <input
      type="text"
      maxlength="128"
      bind:value={search}
      class="focus:border-accent-500 block w-96 rounded-lg border border-neutral-600 bg-neutral-800 px-8 text-sm text-neutral-300 placeholder-neutral-500 focus:ring-0"
      placeholder="Search encounters, names, or class:name pairs"
      oninput={handleSearchInput}
    />
    <button
      class="absolute inset-y-0 right-0 flex items-center pr-2"
      class:hidden={search.length === 0 && !active}
      onclick={onClear}
    >
      <IconX />
    </button>
  </div>
  <div class="flex items-center gap-4">
    {#if selectMode && selected.size > 0}
      <button class="group rounded-md border border-neutral-700 p-1" use:melt={$dialogTrigger}>
        <QuickTooltip tooltip="Delete selected encounters">
          <IconTrash class="text-neutral-300 group-hover:text-red-500/70" />
        </QuickTooltip>
      </button>
    {/if}
    <button
      class="mr-2 rounded-md border border-neutral-700 px-1 py-0.5 {selectMode
        ? 'bg-accent-500/80'
        : 'bg-neutral-800/80 hover:bg-neutral-700/70'}"
      onclick={onClearSelected}>select</button
    >
  </div>
</div>

{#if $open}
  <div
    use:melt={$menu}
    class="z-20 flex max-h-80 w-96 flex-col rounded-md border border-neutral-600 bg-neutral-800/80 text-sm text-neutral-200 shadow-lg backdrop-blur-lg {settings
      .app.general.accentColor}"
    transition:fly={{ duration: 150, y: -10 }}
  >
    <div class="sticky top-0 flex items-center justify-between gap-2 rounded-t bg-neutral-900">
      <div class="flex items-center">
        {@render tab("Encounters")}
        {@render tab("Bosses")}
        {@render tab("Classes")}
      </div>
      <button
        type="button"
        class="hover:text-accent-500 px-2 {active ? 'text-accent-500' : ''}"
        onclick={onClear}
      >
        reset
      </button>
    </div>
    {#if currentTab === "Encounters"}
      <div class="flex flex-col gap-1 overflow-y-auto overflow-x-hidden py-1 text-xs">
        <div class="flex items-center space-x-4 px-3 py-1">
          <label class="flex items-center">
            <div class="mr-2">Raid Cleared</div>
            <input
              type="checkbox"
              bind:checked={$encounterFilter.cleared}
              class="form-checkbox checked:text-accent-500 size-4 rounded-sm bg-neutral-700 focus:ring-0 focus:ring-offset-0"
            />
          </label>
          <label class="flex items-center">
            <div class="mr-2">Favorites</div>
            <input
              type="checkbox"
              bind:checked={$encounterFilter.favorite}
              class="form-checkbox checked:text-accent-500 size-4 rounded-sm bg-neutral-700 focus:ring-0 focus:ring-offset-0"
            />
          </label>
        </div>
        <div class="flex flex-wrap px-1">
          {#each difficultyMap as difficulty}
            <button
              type="button"
              data-difficulty={difficulty}
              data-selected={$encounterFilter.difficulty === difficulty}
              class="m-1 rounded border border-neutral-700 px-1 {$encounterFilter.difficulty === difficulty
                ? 'bg-neutral-700'
                : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
              onclick={onDifficultySelect}
            >
              {difficulty}
            </button>
          {/each}
        </div>
        <div class="mx-2 h-px bg-neutral-600">&nbsp</div>
        <div class="flex flex-col px-1">
          {#each Object.entries(encounterMap).reverse() as raid}
            <div class="flex flex-wrap">
              {#each Object.keys(raid[1]) as encounter}
                <button
                  type="button"
                  data-encounter={encounter}
                  data-selected={$encounterFilter.encounters.has(encounter)}
                  class="m-1 rounded border border-neutral-700 p-1 {$encounterFilter.encounters.has(encounter)
                    ? 'bg-neutral-700'
                    : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
                  onclick={onEncounterSelect}
                >
                  {encounter}
                </button>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {:else if currentTab === "Bosses"}
      <div class="flex flex-wrap px-1 py-2 text-xs">
        {#each bossList as boss}
          <button
            type="button"
            data-boss={boss}
            data-selected={$encounterFilter.bosses.has(boss)}
            class="m-1 rounded border border-neutral-700 p-1 {$encounterFilter.bosses.has(boss)
              ? 'bg-neutral-700'
              : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
            onclick={onBossSelect}
          >
            {boss}
          </button>
        {/each}
      </div>
    {:else if currentTab === "Classes"}
      <div class="flex flex-wrap px-1 py-2 text-xs">
        {#each classList.sort() as className (className)}
          <button
            type="button"
            data-classname={className}
            data-selected={search.includes(className)}
            class="m-1 rounded border border-neutral-700 p-1"
            onclick={onClassSelect}
          >
            {className}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

{#if $dialogOpen}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-[450px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800 p-4 shadow-lg
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="font-semibold">Delete Encounters</h2>
      <p use:melt={$description} class="text-center">
        Are you sure you want to delete the {selected.size} selected encounter(s)? This action is irreversible.
      </p>
      <div class="flex items-center gap-28 pt-5">
        <button type="button" use:melt={$close} class="rounded-md bg-neutral-700 p-1 hover:bg-neutral-700/80"> Close </button>
        <button
          type="button"
          use:melt={$close}
          class="rounded-md bg-red-500/70 p-1 hover:bg-red-500/60"
          onclick={onDelete}
        >
          Confirm
        </button>
      </div>
    </div>
  </div>
{/if}
