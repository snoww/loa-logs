<script lang="ts">
  import { bossList } from "$lib/constants/bosses";
  import { classList } from "$lib/constants/classes";
  import { difficultyMap, encounterMap } from "$lib/constants/encounters";
  import { IconFilter, IconX } from "$lib/icons";
  import { encounterFilter, settings } from "$lib/stores.svelte";
  import { createDropdownMenu, melt } from "@melt-ui/svelte";
  import type { FormEventHandler } from "svelte/elements";
  import { fly } from "svelte/transition";

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

  let currentTab = $state("Encounters");
  let search = $state(encounterFilter.search || "");
  let active = $derived(
    encounterFilter.encounters.size > 0 ||
      encounterFilter.bosses.size > 0 ||
      encounterFilter.cleared ||
      encounterFilter.favorite ||
      encounterFilter.difficulty !== "" ||
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

  const handleSearchInput = debounce(() => {
    encounterFilter.search = search.length >= 1 ? search : "";
  }, 300);
</script>

{#snippet tab(tab: string)}
  <button
    class="hover:text-accent-500 p-2 first:rounded-tl {currentTab === tab ? 'text-accent-500 bg-neutral-800' : ''}"
    onclick={() => (currentTab = tab)}
  >
    {tab}
  </button>
{/snippet}

<div class="flex items-center justify-between py-1">
  <div class="relative flex items-center gap-2">
    <button class="hover:text-accent-500 absolute left-2.5" use:melt={$trigger}>
      <IconFilter class={active ? "text-accent-500" : ""} />
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
      class:hidden={search.length === 0}
      onclick={() => {
        search = "";
        encounterFilter.search = "";
      }}
    >
      <IconX />
    </button>
  </div>
  <div>select mode</div>
</div>

{#if $open}
  <div
    use:melt={$menu}
    class="z-20 flex max-h-80 w-96 flex-col rounded-md border border-neutral-600 bg-neutral-800/80 text-sm text-neutral-200 shadow-lg backdrop-blur-lg {settings
      .appSettings.general.accentColor}"
    transition:fly={{ duration: 150, y: -10 }}
  >
    <div class="sticky top-0 flex items-center justify-between gap-2 rounded-t bg-neutral-900">
      <div class="flex items-center">
        {@render tab("Encounters")}
        {@render tab("Bosses")}
        {@render tab("Classes")}
      </div>
      <button
        class="hover:text-accent-500 px-2"
        onclick={() => {
          search = "";
          encounterFilter.reset();
        }}
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
              bind:checked={encounterFilter.cleared}
              class="form-checkbox checked:text-accent-500 size-4 rounded-sm bg-neutral-700 focus:ring-0 focus:ring-offset-0"
            />
          </label>
          <label class="flex items-center">
            <div class="mr-2">Favorites</div>
            <input
              type="checkbox"
              bind:checked={encounterFilter.favorite}
              class="form-checkbox checked:text-accent-500 size-4 rounded-sm bg-neutral-700 focus:ring-0 focus:ring-offset-0"
            />
          </label>
        </div>
        <div class="flex flex-wrap px-1">
          {#each difficultyMap as difficulty}
            <button
              class="m-1 rounded border border-neutral-700 px-1 {encounterFilter.difficulty === difficulty
                ? 'bg-neutral-700'
                : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
              onclick={() => {
                encounterFilter.difficulty = encounterFilter.difficulty === difficulty ? "" : difficulty;
              }}
            >
              {difficulty}
            </button>
          {/each}
        </div>
        <div class="mx-2 h-px bg-neutral-600">&nbsp</div>
        <div class="flex flex-col px-1">
          {#each Object.entries(encounterMap).reverse() as raid}
            <div class="flex flex-wrap">
              {#each Object.keys(raid[1]) as encounter (encounter)}
                <button
                  class="m-1 rounded border border-neutral-700 p-1 {encounterFilter.encounters.has(encounter)
                    ? 'bg-neutral-700'
                    : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
                  onclick={() => {
                    encounterFilter.encounters.has(encounter)
                      ? encounterFilter.encounters.delete(encounter)
                      : encounterFilter.encounters.add(encounter);
                    encounterFilter.encounters = new Set(encounterFilter.encounters);
                  }}
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
            class="m-1 rounded border border-neutral-700 p-1 {encounterFilter.bosses.has(boss)
              ? 'bg-neutral-700'
              : 'bg-neutral-800/80 hover:bg-neutral-700/80'}"
            onclick={() => {
              encounterFilter.bosses.has(boss) ? encounterFilter.bosses.delete(boss) : encounterFilter.bosses.add(boss);
              encounterFilter.bosses = new Set(encounterFilter.bosses);
            }}
          >
            {boss}
          </button>
        {/each}
      </div>
    {:else if currentTab === "Classes"}
      <div class="flex flex-wrap px-1 py-2 text-xs">
        {#each classList.sort() as className (className)}
          <button
            class="m-1 rounded border border-neutral-700 p-1"
            onclick={() => {
              search += ` ${className.toLowerCase()}:`;
              encounterFilter.search = search;
            }}
          >
            {className}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}
