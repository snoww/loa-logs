<script lang="ts">
    import { onMount } from "svelte";

    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import TableFilter from "$lib/components/table/TableFilter.svelte";
    import type { EncounterPreview, EncountersOverview, SearchFilter } from "$lib/types";
    import {
        abbreviateNumber,
        formatDurationFromMs,
        formatTimestamp,
        formatTimestampDate,
        formatTimestampTime
    } from "$lib/utils/numbers";
    import { classIconCache, miscSettings, settings } from "$lib/utils/settings";
    import {
        backNavStore,
        ifaceChangedStore,
        pageStore,
        raidGates,
        searchFilter,
        searchStore,
        selectedEncounters
    } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api";
    import NProgress from "nprogress";
    import { goto } from "$app/navigation";
    import "nprogress/nprogress.css";
    import Notification from "$lib/components/shared/Notification.svelte";
    import { classNameToClassId } from "$lib/constants/classes";
    import { encounterMap } from "$lib/constants/encounters";
    import DifficultyLabel from "$lib/components/shared/DifficultyLabel.svelte";
    import SortSymbol from "$lib/components/table/SortSymbol.svelte";
    import Title from "$lib/components/shared/Title.svelte";
    import { getVersion } from "@tauri-apps/api/app";
    import { appWindow } from "@tauri-apps/api/window";

    let encounters: Array<EncounterPreview> = [];
    let totalEncounters: number = 0;
    let selectMode = false;

    $: {
        if ($settings.general.logsPerPage <= 0 || $settings.general.logsPerPage > 100) {
            $settings.general.logsPerPage = 10;
        }

        if ($searchStore.length > 0) {
            if ($backNavStore) {
                $backNavStore = false;
            } else {
                $pageStore = 1;
            }
        }
    }

    // Initialize `minDuration` here to not trigger a reload of encounters
    if ($searchFilter.minDuration === -1) {
        $searchFilter.minDuration = $settings.logs.minEncounterDuration;
    }
    $: loadEncounters($searchFilter, $searchStore, $pageStore);

    onMount(async () => {
        if ($settings.general.logsPerPage <= 0 || $settings.general.logsPerPage > 100) {
            $settings.general.logsPerPage = 10;
        }
        if ($miscSettings) {
            const version = await getVersion();
            if (!$miscSettings.viewedChangelog || $miscSettings.version !== version) {
                $miscSettings.version = version;
                await gotoChangelog();
            }
        } else {
            $miscSettings = { version: await getVersion() };
            await gotoChangelog();
        }
    });

    async function gotoChangelog() {
        await appWindow.show();
        await appWindow.unminimize();
        await appWindow.setFocus();

        goto("/changelog");
    }

    async function loadEncounters(
        searchFilter: SearchFilter,
        search: string,
        page: number
    ): Promise<Array<EncounterPreview>> {
        NProgress.start();
        let bosses = Array.from($searchFilter.bosses);
        if (searchFilter.encounters.size > 0) {
            for (const encounter of searchFilter.encounters) {
                const raid = encounter.substring(0, encounter.indexOf(" "));
                bosses.push(...encounterMap[raid][encounter]);
            }
        }
        // start or space (^|\s) + word (\w+) + colon or space or end (:|\s|$)
        // using lookbehind (?<=) and lookahead (?=) https://regex101.com/r/1cMFH8/4
        // if word is a valid className, replace it with the classId
        // example: "bard:Anyduck shadowhunter" -> "204:Anyduck 403"
        let searchQuery = search.replace(/(?<=^|\s)\w+(?=:|\s|$)/g, (word: string) => {
            const className = word[0].toUpperCase() + word.substring(1).toLowerCase();
            return String(classNameToClassId[className] || word);
        });

        let overview: EncountersOverview = await invoke("load_encounters_preview", {
            page: page,
            pageSize: $settings.general.logsPerPage,
            search: searchQuery,
            filter: {
                minDuration: searchFilter.minDuration,
                bosses: bosses,
                cleared: searchFilter.cleared,
                favorite: searchFilter.favorite,
                difficulty: searchFilter.difficulty,
                bossOnlyDamage: searchFilter.bossOnlyDamage,
                sort: searchFilter.sort,
                order: searchFilter.order
            }
        });
        encounters = overview.encounters;
        totalEncounters = overview.totalEncounters;
        NProgress.done();
        return encounters;
    }

    function refresh() {
        $pageStore = 1;
        $backNavStore = false;
        $searchFilter = $searchFilter;
    }

    function nextPage() {
        if ($pageStore * $settings.general.logsPerPage < totalEncounters) {
            $pageStore++;
            scrollToTopOfTable();
        }
    }

    function previousPage() {
        if ($pageStore > 1) {
            $pageStore--;
            scrollToTopOfTable();
        }
    }

    function firstPage() {
        $pageStore = 1;
        scrollToTopOfTable();
    }

    function lastPage() {
        $pageStore = Math.ceil(totalEncounters / $settings.general.logsPerPage);
        scrollToTopOfTable();
    }

    // 0: none, 1: asc, 2: desc
    function setSort(sort: string) {
        let order = $searchFilter.sort === sort ? ($searchFilter.order + 1) % 3 : 1;

        if (order === 0) {
            $searchFilter.sort = "id";
            $searchFilter.order = sort === "id" ? 1 : 2;
        } else {
            $searchFilter.sort = sort;
            $searchFilter.order = order;
        }
        $pageStore = 1;
    }

    function scrollToTopOfTable() {
        if (encounters.length === 0) {
            return;
        }
        const rows = document.querySelectorAll(`#encounter-${encounters[0].id}`);
        rows[0].scrollIntoView({
            behavior: "smooth",
            block: "center"
        });
    }

    async function changeRowsPerPage(event: Event) {
        $settings.general.logsPerPage = parseInt((event.target as HTMLSelectElement).value);
        $pageStore = 1;
        await loadEncounters($searchFilter, $searchStore, $pageStore);
        scrollToTopOfTable();
    }

    let hidden: boolean = true;
</script>

<svelte:window on:contextmenu|preventDefault />
<LogSidebar bind:hidden />
<div class="h-screen bg-zinc-800">
    <div class="flex h-16 items-center justify-between px-8 py-5 shadow-md">
        <Title text="Past Encounters" bind:hidden />
        <button
            class="bg-accent-900 hover:bg-accent-800 mr-4 rounded-md px-2 py-1 shadow-md"
            on:click={() => refresh()}>
            Refresh
        </button>
    </div>
    <div class="px-8">
        <div class="py-2">
            <TableFilter bind:selectMode refreshFn={refresh} />
        </div>
        <div
            class="relative overflow-y-auto overflow-x-hidden"
            style="height: calc(100vh - 8.25rem - 2.5rem);"
            id="logs-table">
            <table class="w-full table-fixed text-left text-gray-400" id="table">
                <thead class="sticky top-0 z-10 bg-zinc-900 text-xs uppercase">
                    <tr>
                        {#if selectMode}
                            <th scope="col" class="w-14 px-2 py-3">
                                <input
                                    type="checkbox"
                                    class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0"
                                    checked={encounters.every((encounter) => $selectedEncounters.has(encounter.id))}
                                    on:change={() => {
                                        if (encounters.every((encounter) => $selectedEncounters.has(encounter.id))) {
                                            selectedEncounters.update((set) => {
                                                encounters.forEach((encounter) => {
                                                    set.delete(encounter.id);
                                                });
                                                return set;
                                            });
                                        } else {
                                            selectedEncounters.update((set) => {
                                                encounters.forEach((encounter) => {
                                                    set.add(encounter.id);
                                                });
                                                return set;
                                            });
                                        }
                                    }} />
                            </th>
                        {:else}
                            <th scope="col" class="w-14 px-3 py-3">
                                <button class="hover:text-accent-500 flex items-center" on:click={() => setSort("id")}>
                                    ID
                                    {#if $searchFilter.sort === "id"}
                                        <SortSymbol />
                                    {/if}
                                </button>
                            </th>
                        {/if}
                        <th scope="col" class="w-[25%] px-3 py-3"> Encounter</th>
                        <th scope="col" class="px-3 py-3"> Classes</th>
                        <th scope="col" class="hidden w-32 px-3 py-3 md:table-cell lg:w-48"> Local Player</th>
                        <th scope="col" class="hidden w-24 px-3 py-3 text-right lg:table-cell">
                            <button
                                class="hover:text-accent-500 ml-auto flex items-center"
                                on:click={() => setSort("my_dps")}>
                                {#if $searchFilter.sort === "my_dps"}
                                    <SortSymbol />
                                {/if}
                                MY DPS
                            </button>
                        </th>
                        <th scope="col" class="w-14 px-3 py-3">
                            <button
                                class="hover:text-accent-500 flex items-center"
                                on:click={() => setSort("duration")}>
                                DUR
                                {#if $searchFilter.sort === "duration"}
                                    {#if $searchFilter.order === 2}
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="ml-1 size-4 fill-gray-400"
                                            viewBox="0 -960 960 960">
                                            <path d="M480-332 233-580h494L480-332Z" />
                                        </svg>
                                    {:else}
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="ml-1 size-4 fill-gray-400"
                                            viewBox="0 -960 960 960">
                                            <path d="M480-580 727-332H233L480-580Z" />
                                        </svg>
                                    {/if}
                                {/if}
                            </button>
                        </th>
                        <th scope="col" class="w-[15%] px-3 py-3">
                            <button
                                class="hover:text-accent-500 ml-auto flex items-center"
                                on:click={() => setSort("fight_start")}>
                                {#if $searchFilter.sort === "fight_start"}
                                    <SortSymbol />
                                {/if}
                                DATE
                            </button>
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-neutral-800 tracking-tight">
                    {#each encounters as encounter (encounter.id)}
                        <tr class="border-b border-gray-700 hover:bg-zinc-700" id="encounter-{encounter.id}">
                            <td class="px-2 py-3">
                                {#if selectMode}
                                    <div>
                                        <input
                                            type="checkbox"
                                            class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0"
                                            checked={$selectedEncounters.has(encounter.id)}
                                            on:change={() => {
                                                if ($selectedEncounters.has(encounter.id)) {
                                                    selectedEncounters.update((set) => {
                                                        set.delete(encounter.id);
                                                        return set;
                                                    });
                                                } else {
                                                    selectedEncounters.update((set) => {
                                                        set.add(encounter.id);
                                                        return set;
                                                    });
                                                }
                                            }} />
                                    </div>
                                {:else}
                                    <div
                                        use:tooltip={{
                                            content:
                                                (encounter.cleared ? "Clear " : "") +
                                                formatTimestamp(encounter.fightStart)
                                        }}>
                                        <div class="" class:text-lime-400={encounter.cleared}>
                                            #{encounter.id}
                                        </div>
                                    </div>
                                {/if}
                            </td>
                            <td class="w-full truncate px-3 py-3 font-medium">
                                <a
                                    href="/logs/encounter/{encounter.id}"
                                    class="hover:text-accent-500 group flex items-center hover:underline"
                                    use:tooltip={{ content: encounter.bossName }}>
                                    {#if encounter.favorite}
                                        <svg
                                            class="mr-1 size-5 flex-shrink-0 fill-yellow-400"
                                            xmlns="http://www.w3.org/2000/svg"
                                            viewBox="0 -960 960 960">
                                            <path
                                                d="m235-82.5 64.5-279.093L83-549l286-25 111-263 111.5 263L877-549 660.484-361.593 725.436-82.5 480.218-230.61 235-82.5Z" />
                                        </svg>
                                    {/if}
                                    <div class="truncate">
                                        {#if encounter.difficulty && $settings.general.showDifficulty}
                                            <DifficultyLabel difficulty={encounter.difficulty} hover={true} />
                                            {@const gate = $raidGates.get(encounter.bossName)}
                                            {#if $settings.general.showGate && gate}
                                                <span class="group-hover:text-accent-500 text-sky-200"> [{gate}]</span>
                                            {/if}
                                            {encounter.bossName}
                                        {:else}
                                            {@const gate = $raidGates.get(encounter.bossName)}
                                            {#if $settings.general.showGate && gate}
                                                <span class="text-sky-200"> [{gate}]</span>
                                            {/if}
                                            {encounter.bossName}
                                        {/if}
                                    </div>
                                </a>
                            </td>
                            <td
                                class="flex truncate px-3 py-3"
                                style="-webkit-mask-image: linear-gradient(to right, black 90%, transparent 100%);">
                                {#each encounter.classes as classId, i}
                                    <img
                                        src={$classIconCache[classId]}
                                        alt="class-{classId}"
                                        class="size-8"
                                        use:tooltip={{ content: encounter.names[i] }} />
                                {/each}
                            </td>
                            <td class="hidden truncate px-3 py-3 md:table-cell">
                                {encounter.localPlayer}
                            </td>
                            <td class="hidden truncate px-3 py-3 lg:table-cell">
                                <div
                                    class="pr-1 text-right"
                                    use:tooltip={{ content: encounter.myDps.toLocaleString() }}>
                                    {abbreviateNumber(encounter.myDps).toUpperCase()}
                                </div>
                            </td>
                            <td class="px-3 py-3">
                                {formatDurationFromMs(encounter.duration)}
                            </td>
                            <td class="px-3 py-3 text-right text-xs">
                                {formatTimestampDate(encounter.fightStart)}
                                {formatTimestampTime(encounter.fightStart)}
                            </td>
                        </tr>
                    {:else}
                        {#if $searchStore.length > 0}
                            <div class="w-screen bg-neutral-800 p-2">No encounters found.</div>
                        {:else}
                            <div class="w-screen bg-neutral-800 p-2">No encounters recorded.</div>
                            <div class="w-screen bg-neutral-800 p-2">
                                Meter should be turned on at character select (before entering raid at latest) for best
                                accuracy.
                            </div>
                        {/if}
                    {/each}
                </tbody>
            </table>
        </div>
        {#if encounters.length > 0}
            <div class="flex items-center justify-between py-3">
                <div class="flex items-center gap-2">
                    <label for="rowsPerPage" class="text-sm text-gray-400">Rows per page:</label>
                    <select
                        id="rowsPerPage"
                        class="focus:border-accent-500 inline rounded-lg border border-gray-600 bg-zinc-700 px-1 py-1 text-sm text-zinc-300 placeholder-gray-400 focus:ring-0"
                        on:change={changeRowsPerPage}>
                        <option selected={$settings.general.logsPerPage === 10}>10</option>
                        <option selected={$settings.general.logsPerPage === 25}>25</option>
                        <option selected={$settings.general.logsPerPage === 50}>50</option>
                        <option selected={$settings.general.logsPerPage === 100}>100</option>
                    </select>

                    <span class="text-sm text-gray-400"
                        >Showing <span class="font-semibold dark:text-white"
                            >{($pageStore - 1) * $settings.general.logsPerPage + 1}-{Math.min(
                                ($pageStore - 1) * $settings.general.logsPerPage + 1 + $settings.general.logsPerPage - 1,
                                totalEncounters
                            )}</span>
                        of
                        <span class="font-semibold text-white">{totalEncounters === 0 ? 1 : totalEncounters}</span
                        ></span>
                </div>
                <ul class="inline-flex items-center -space-x-px">
                    <li use:tooltip={{ content: "First" }}>
                        <button class="ml-0 block px-3" on:click={() => firstPage()}>
                            <span class="sr-only">First</span>
                            <svg
                                class="hover:fill-accent-800 size-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960">
                                <path
                                    d="M226 837V314.5h91.5V837H226Zm459.5-3.5L431 579l254.5-254.5 65.5 65L561.5 579 751 768.5l-65.5 65Z" />
                            </svg>
                        </button>
                    </li>
                    <li use:tooltip={{ content: "Previous" }}>
                        <button class="ml-0 block px-3" on:click={() => previousPage()}>
                            <span class="sr-only">Back</span>
                            <svg
                                class="hover:fill-accent-800 size-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960">
                                <path d="m560.5 837-262-262 262-262 65 65.5L429 575l196.5 196.5-65 65.5Z" />
                            </svg>
                        </button>
                    </li>
                    <li use:tooltip={{ content: "Next" }}>
                        <button class="ml-0 block px-3" on:click={() => nextPage()}>
                            <span class="sr-only">Next</span>
                            <svg
                                class="hover:fill-accent-800 size-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960">
                                <path d="m375.5 837-65-65.5L507 575 310.5 378.5l65-65.5 262 262-262 262Z" />
                            </svg>
                        </button>
                    </li>
                    <li use:tooltip={{ content: "Last" }}>
                        <button class="ml-0 block px-3" on:click={() => lastPage()}>
                            <span class="sr-only">Last</span>
                            <svg
                                class="hover:fill-accent-800 size-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960">
                                <path
                                    d="m273.5 831.5-65.5-65 191-191-191-191 65.5-65 256 256-256 256ZM643 837V314.5h91.5V837H643Z" />
                            </svg>
                        </button>
                    </li>
                </ul>
            </div>
        {/if}
    </div>
    {#if $ifaceChangedStore}
        <Notification
            bind:showAlert={$ifaceChangedStore}
            text={"Network Interface Changed. Please fully Restart the App."}
            dismissable={false}
            width="18rem"
            isError={true} />
    {/if}
</div>
