<script lang="ts">
    import { bossList } from "$lib/constants/bosses";
    import { classList } from "$lib/constants/classes";
    import { SearchFilter } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { pageStore, searchStore, searchFilter } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { onMount } from "svelte";

    let filterMenu = false;
    let filterTab = "Encounters";

    let filterDiv: HTMLDivElement;


    onMount(() => {
        if ($searchFilter.minDuration === -1) {
            $searchFilter.minDuration = $settings.logs.minEncounterDuration;
        }

        const clickOutside = (event: MouseEvent) => {
            if (!filterDiv.contains(event.target as Node)) {
                filterMenu = false;
            }
        };
        document.addEventListener("click", clickOutside);
        return () => {
            document.removeEventListener("click", clickOutside);
        };
    });
</script>

<div class="flex items-center justify-between">
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
                            class="h-5 w-5 {$searchFilter.bossFilter.size > 0 ||
                            $searchFilter.classFilter.size > 0 ||
                            $searchFilter.minDuration !== $settings.logs.minEncounterDuration
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
                                        searchFilter.set(new SearchFilter($settings.logs.minEncounterDuration));
                                    }}>
                                    Reset All
                                </button>
                            </div>
                            {#if filterTab === "Encounters"}
                                <div class="flex h-36 flex-wrap overflow-auto px-2 py-1 text-xs">
                                    {#each bossList as boss (boss)}
                                        <button
                                            class="m-1 truncate rounded border border-gray-500 p-1 {$searchFilter.bossFilter.has(
                                                boss
                                            )
                                                ? 'bg-gray-800'
                                                : ''}"
                                            on:click={() => {
                                                let newSet = new Set($searchFilter.bossFilter);
                                                if (newSet.has(boss)) {
                                                    newSet.delete(boss);
                                                } else {
                                                    newSet.add(boss);
                                                }
                                                $searchFilter.bossFilter = newSet;
                                            }}>
                                            {boss}
                                        </button>
                                    {/each}
                                </div>
                            {:else if filterTab === "Classes"}
                                <div class="flex h-36 flex-wrap overflow-auto px-2 py-1 text-xs">
                                    {#each classList.sort() as className (className)}
                                        <button
                                            class="m-1 truncate rounded border border-gray-500 p-1 {$searchFilter.classFilter.has(
                                                className
                                            )
                                                ? 'bg-gray-800'
                                                : ''}"
                                            on:click={() => {
                                                let newSet = new Set($searchFilter.classFilter);
                                                if (newSet.has(className)) {
                                                    newSet.delete(className);
                                                } else {
                                                    newSet.add(className);
                                                }
                                                $searchFilter.classFilter = newSet;
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
                placeholder="Search encounters, names, or classes" />
            {#if $searchStore.length > 0}
                <button
                    class="absolute inset-y-0 right-0 flex items-center pr-2"
                    on:click={() => {
                        searchStore.set("");
                        pageStore.set(1);
                        $searchFilter = new SearchFilter($settings.logs.minEncounterDuration);
                    }}>
                    <svg
                        class="h-5 w-5 fill-gray-400 hover:fill-gray-200"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960"
                        ><path
                            d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" /></svg>
                </button>
            {/if}
        </div>
    </div>
    <div />
</div>
