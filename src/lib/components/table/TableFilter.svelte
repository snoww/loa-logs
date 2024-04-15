<script lang="ts">
    import { onMount } from "svelte";
    import type { FormEventHandler } from "svelte/elements";

    import { bossList } from "$lib/constants/bosses";
    import { classList } from "$lib/constants/classes";
    import { difficultyMap, encounterMap } from "$lib/constants/encounters";
    import { SearchFilter, type EncounterPreview } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { pageStore, searchStore, searchFilter, selectedEncounters } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api";

    let filterMenu = false;
    let filterTab = "Encounters";

    let filterDiv: HTMLDivElement;

    export let selectMode: boolean;
    export let refreshFn: () => void;
    export let loadEncountersFn: () => Promise<Array<EncounterPreview>>;

    let deleteConfirm = false;

    onMount(() => {
        if ($searchFilter.minDuration === -1) {
            $searchFilter.minDuration = $settings.logs.minEncounterDuration;
        }

        const clickOutside = (event: MouseEvent) => {
            if (filterDiv && filterDiv.contains(event.target as Node)) {
                return;
            }
            if (isFilterButton(event.target as HTMLElement)) {
                return;
            }

            filterMenu = false;
        };
        document.addEventListener("click", clickOutside);
        return () => {
            document.removeEventListener("click", clickOutside);
        };
    });

    function debounce(fn: FormEventHandler<HTMLInputElement>, milliseconds: number) {
        let timer: number | undefined;

        if ($searchStore.length === 0) {
            return fn;
        }

        return (evt: Event & { currentTarget: EventTarget & HTMLInputElement }) => {
            clearTimeout(timer);
            timer = setTimeout(() => {
                fn(evt);
            }, milliseconds);
        };
    }

    const handleSearchInput = debounce((e) => {
        loadEncountersFn();
    }, 500);

    const isFilterButton = (element: HTMLElement) => {
        return element.classList.contains("filter-button");
    };

    function toggleSelectMode() {
        selectMode = !selectMode;
        if (!selectMode) {
            selectedEncounters.set(new Set());
        }
    }

    async function deleteSelected() {
        await invoke("delete_encounters", { ids: Array.from($selectedEncounters) });
        deleteConfirm = false;
        selectMode = false;
        $selectedEncounters = new Set();
        setTimeout(() => {
            refreshFn();
        }, 500);
    }
</script>

<div class="z-30 flex items-center justify-between">
    <div class="flex items-center space-x-2">
        <div class="relative">
            <div class="absolute inset-y-0 left-0 flex cursor-default items-center pl-2">
                <div class="relative flex items-center">
                    <button
                        use:tooltip={{ content: "Search Filter" }}
                        on:click|stopPropagation={() => {
                            filterMenu = !filterMenu;
                        }}>
                        <svg
                            class="size-5 {$searchFilter.bosses.size > 0 ||
                            $searchFilter.encounters.size > 0 ||
                            $searchFilter.difficulty ||
                            $searchFilter.classes.size > 0 ||
                            $searchFilter.favorite ||
                            $searchFilter.bossOnlyDamage ||
                            $searchFilter.minDuration !== $settings.logs.minEncounterDuration ||
                            $searchFilter.cleared
                                ? 'fill-accent-500'
                                : 'fill-gray-400 hover:fill-gray-200'}"
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 -960 960 960"
                            ><path
                                d="M420.5-101v-244.5H501v82.5h352.5v80.5H501v81.5h-80.5Zm-314-81.5V-263H375v80.5H106.5Zm188-177v-81h-188V-520h188v-83.5H375v244h-80.5Zm126-81V-520h433v79.5h-433Zm164.5-175V-860h80.5v81.5h188v80.5h-188v82.5H585ZM106.5-698v-80.5h433v80.5h-433Z" /></svg>
                    </button>
                    {#if filterMenu}
                        <div
                            class="absolute -left-2 top-9 z-40 h-44 w-96 rounded bg-zinc-700 shadow-lg"
                            bind:this={filterDiv}>
                            <div class="flex items-center justify-between shadow-md">
                                <div class="mx-2 my-1 flex items-center space-x-2">
                                    <button
                                        class="border-b px-1 {filterTab === 'Encounters'
                                            ? 'border-zinc-200'
                                            : 'border-zinc-700 text-gray-400'}"
                                        on:click={() => {
                                            filterTab = "Encounters";
                                        }}>
                                        Encounters
                                    </button>
                                    <button
                                        class="border-b px-1 {filterTab === 'Bosses'
                                            ? 'border-zinc-200'
                                            : 'border-zinc-700 text-gray-400'}"
                                        on:click={() => {
                                            filterTab = "Bosses";
                                        }}>
                                        Bosses
                                    </button>
                                    <button
                                        class="border-b px-1 {filterTab === 'Classes'
                                            ? 'border-zinc-200'
                                            : 'border-zinc-700 text-gray-400'}"
                                        on:click={() => {
                                            filterTab = "Classes";
                                        }}>
                                        Classes
                                    </button>
                                    <button
                                        class="border-b px-1 {filterTab === 'Duration'
                                            ? 'border-zinc-200'
                                            : 'border-zinc-700 text-gray-400'}"
                                        on:click={() => {
                                            filterTab = "Duration";
                                        }}>
                                        Duration
                                    </button>
                                </div>
                                <button
                                    class="mx-2 rounded bg-zinc-800 px-1 text-xs hover:bg-zinc-600"
                                    on:click={() => {
                                        let sf = new SearchFilter($settings.logs.minEncounterDuration);
                                        sf.sort = $searchFilter.sort;
                                        sf.order = $searchFilter.order;
                                        searchFilter.set(sf);
                                        $pageStore = 1;
                                    }}>
                                    Reset All
                                </button>
                            </div>
                            {#if filterTab === "Encounters"}
                                <div class="h-36 overflow-auto px-2 py-1 text-xs">
                                    <div class="flex items-center space-x-4 px-2 py-1 text-xs">
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Raid Cleared</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.cleared}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Favorites</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.favorite}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Boss Only</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.bossOnlyDamage}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                    </div>
                                    <div class="flex flex-wrap">
                                        {#each difficultyMap as difficulty (difficulty)}
                                            <button
                                                class="m-1 truncate rounded border border-gray-500 px-1 {$searchFilter.difficulty ===
                                                difficulty
                                                    ? 'bg-gray-800'
                                                    : ''}"
                                                on:click={() => {
                                                    if ($searchFilter.difficulty === difficulty) {
                                                        $searchFilter.difficulty = "";
                                                    } else {
                                                        $searchFilter.difficulty = difficulty;
                                                    }
                                                    $pageStore = 1;
                                                }}>
                                                {difficulty}
                                            </button>
                                        {/each}
                                    </div>
                                    <div class="flex flex-col">
                                        {#each Object.entries(encounterMap).reverse() as raid (raid)}
                                            <div class="flex flex-wrap">
                                                {#each Object.keys(raid[1]) as encounter (encounter)}
                                                    <button
                                                        class="filter-button m-1 truncate rounded border border-gray-500 p-1 {$searchFilter.encounters.has(
                                                            encounter
                                                        )
                                                            ? 'bg-gray-800'
                                                            : ''}"
                                                        on:click={() => {
                                                            let newSet = new Set($searchFilter.encounters);
                                                            if (newSet.has(encounter)) {
                                                                newSet.delete(encounter);
                                                            } else {
                                                                newSet.add(encounter);
                                                            }
                                                            $searchFilter.encounters = newSet;
                                                            $pageStore = 1;
                                                        }}>
                                                        {encounter}
                                                    </button>
                                                {/each}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {:else if filterTab === "Bosses"}
                                <div class="h-36 overflow-auto px-2 py-1 text-xs">
                                    <div class="flex items-center space-x-4 px-2 py-1 text-xs">
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Raid Cleared</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.cleared}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Favorites</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.favorite}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                        <label class="flex items-center">
                                            <div class="mr-2 text-gray-100">Boss Only</div>
                                            <input
                                                type="checkbox"
                                                bind:checked={$searchFilter.bossOnlyDamage}
                                                class="text-accent-500 size-4 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                        </label>
                                    </div>
                                    <div class="flex flex-wrap">
                                        {#each bossList as boss (boss)}
                                            <button
                                                class="m-1 truncate rounded border border-gray-500 p-1 {$searchFilter.bosses.has(
                                                    boss
                                                )
                                                    ? 'bg-gray-800'
                                                    : ''}"
                                                on:click={() => {
                                                    let newSet = new Set($searchFilter.bosses);
                                                    if (newSet.has(boss)) {
                                                        newSet.delete(boss);
                                                    } else {
                                                        newSet.add(boss);
                                                    }
                                                    $searchFilter.bosses = newSet;
                                                    $pageStore = 1;
                                                }}>
                                                {boss}
                                            </button>
                                        {/each}
                                    </div>
                                </div>
                            {:else if filterTab === "Classes"}
                                <div class="flex h-36 flex-wrap overflow-auto px-2 py-1 text-xs">
                                    {#each classList.sort() as className (className)}
                                        <button
                                            class="m-1 truncate rounded border border-gray-500 p-1 {$searchFilter.classes.has(
                                                className
                                            )
                                                ? 'bg-gray-800'
                                                : ''}"
                                            on:click={() => {
                                                let newSet = new Set($searchFilter.classes);
                                                if (newSet.has(className)) {
                                                    newSet.delete(className);
                                                } else {
                                                    newSet.add(className);
                                                }
                                                $searchFilter.classes = newSet;
                                                $pageStore = 1;
                                            }}>
                                            {className}
                                        </button>
                                    {/each}
                                </div>
                            {:else if filterTab === "Duration"}
                                <div class="flex h-36 flex-wrap overflow-auto px-2 py-1 text-xs">
                                    <div class="w-96 p-2">
                                        <div class="flex items-center justify-between">
                                            <label class="flex items-center font-medium">
                                                <div class="mr-2">
                                                    <div class="text-gray-100">Min Duration:</div>
                                                </div>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    class="h-6 w-20 rounded-md bg-zinc-700 text-xs text-gray-300"
                                                    bind:value={$searchFilter.minDuration}
                                                    placeholder={$settings.logs.minEncounterDuration} />
                                                <div class="ml-2">seconds</div>
                                            </label>
                                            <button
                                                class="mx-2 h-6 rounded bg-zinc-800 px-1 text-xs hover:bg-zinc-600"
                                                on:click={() => {
                                                    $searchFilter.minDuration = $settings.logs.minEncounterDuration;
                                                }}>
                                                Reset
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            </div>
            <input
                type="text"
                bind:value={$searchStore}
                class="focus:border-accent-500 block w-80 rounded-lg border border-gray-600 bg-zinc-700 px-8 text-sm text-zinc-300 placeholder-gray-400 focus:ring-0"
                placeholder="Search encounters, names, or classes"
                on:input={handleSearchInput} />
            {#if $searchStore.length > 0}
                <button
                    class="absolute inset-y-0 right-0 flex items-center pr-2"
                    on:click={() => {
                        searchStore.set("");
                        pageStore.set(1);
                        $searchFilter = new SearchFilter($settings.logs.minEncounterDuration);
                    }}>
                    <svg
                        class="size-5 fill-gray-400 hover:fill-gray-200"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960"
                        ><path
                            d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" /></svg>
                </button>
            {/if}
        </div>
    </div>
    <div class="flex items-center space-x-2">
        {#if selectMode && $selectedEncounters.size > 0}
            <button
                class="flex items-center rounded-md bg-red-900 p-1 text-xs"
                on:click={() => {
                    deleteConfirm = true;
                }}>
                <svg class="size-5 fill-zinc-300" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                    ><path
                        d="M254.5-100q-39.181 0-65.841-26.366Q162-152.731 162-191.5v-549h-57.5v-91.333H332V-879h295.5v47H856v91.5h-57.5v549q0 38.019-27.034 64.759Q744.431-100 706-100H254.5ZM706-740.5H254.5v549H706v-549ZM356.5-269H431v-396.5h-74.5V-269Zm173 0H605v-396.5h-75.5V-269Zm-275-471.5v549-549Z" /></svg>
            </button>
        {/if}
        <button
            class="flex items-center rounded-md p-1 text-xs {selectMode ? 'bg-accent-800' : 'bg-zinc-700'}"
            on:click={toggleSelectMode}>
            <svg class="size-5 fill-zinc-300" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                ><path
                    d="M191-99.5q-37.744 0-64.622-26.878T99.5-191v-578q0-38.156 26.878-65.328Q153.256-861.5 191-861.5h578q15.545 0 34.773 8.5Q823-844.5 833-836l-71 71v-4H191v578h578v-330l92.5-92.5V-191q0 37.744-27.172 64.622T769-99.5H191ZM467-296 247-516l48-48.5 172.158 172 392.342-392 47 49.5L467-296Z" /></svg>
            <div class="px-1">Select</div>
        </button>
    </div>
</div>

{#if deleteConfirm}
    <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80" />
    <div class="fixed left-0 right-0 top-0 z-50 h-modal w-full items-center justify-center p-4">
        <div class="relative top-[25%] mx-auto flex max-h-full w-full max-w-md">
            <div class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                <button
                    type="button"
                    class="absolute right-2.5 top-3 ml-auto whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 focus:outline-none"
                    aria-label="Close modal"
                    on:click={() => (deleteConfirm = false)}>
                    <span class="sr-only">Close modal</span>
                    <svg class="size-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"
                        ><path
                            fill-rule="evenodd"
                            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                            clip-rule="evenodd" /></svg>
                </button>
                <div id="modal" class="flex-1 space-y-6 overflow-y-auto overscroll-contain p-6">
                    <div class="text-center">
                        <svg
                            aria-hidden="true"
                            class="mx-auto mb-4 h-14 w-14 text-gray-200"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            xmlns="http://www.w3.org/2000/svg"
                            ><path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                class="s-Qbr4I8QhaoSZ" /></svg>
                        <h3 class="mb-5 text-lg font-normal text-gray-400">
                            Are you sure you want to delete {$selectedEncounters.size} encounters?
                        </h3>
                        <button
                            type="button"
                            class="mr-2 inline-flex items-center justify-center rounded-lg bg-red-700 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-red-800 focus:outline-none"
                            on:click={deleteSelected}>
                            Yes, I'm sure
                        </button>
                        <button
                            type="button"
                            class="inline-flex items-center justify-center rounded-lg bg-gray-800 bg-transparent px-5 py-2.5 text-center text-sm font-medium text-gray-400 hover:bg-zinc-700 hover:text-white focus:text-white focus:outline-none"
                            on:click={() => (deleteConfirm = false)}>
                            No, cancel
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
