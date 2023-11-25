<script lang="ts">
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import TableFilter from "$lib/components/table/TableFilter.svelte";
    import type { EncounterPreview, EncountersOverview } from "$lib/types";
    import {
        formatDurationFromMs,
        formatTimestamp,
        formatTimestampDate,
        formatTimestampTime
    } from "$lib/utils/numbers";
    import { classIconCache, settings } from "$lib/utils/settings";
    import {
        backNavStore,
        ifaceChangedStore,
        pageStore,
        searchFilter,
        searchStore,
        selectedEncounters
    } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api";
    import NProgress from "nprogress";
    import "nprogress/nprogress.css";
    import Notification from "$lib/components/shared/Notification.svelte";
    import { encounterMap } from "$lib/constants/encounters";
    import DifficultyLabel from "$lib/components/shared/DifficultyLabel.svelte";
    import BossOnlyDamage from "$lib/components/shared/BossOnlyDamage.svelte";

    let encounters: Array<EncounterPreview> = [];
    let totalEncounters: number = 0;
    const rowsPerPage = 10;
    const maxSearchLength = 30;

    let selectMode = false;

    $: {
        if ($searchStore.length > 0) {
            if ($backNavStore) {
                $backNavStore = false;
            } else {
                $pageStore = 1;
            }
        }
        $searchFilter = $searchFilter;
        loadEncounters();
    }

    async function loadEncounters(): Promise<Array<EncounterPreview>> {
        NProgress.start();
        let bosses = Array.from($searchFilter.bosses);
        if ($searchFilter.encounters.size > 0) {
            for (const encounter of $searchFilter.encounters) {
                const raid = encounter.substring(0, encounter.indexOf(" "));
                bosses.push(...encounterMap[raid][encounter]);
            }
        }
        let overview: EncountersOverview = await invoke("load_encounters_preview", {
            page: $pageStore,
            pageSize: rowsPerPage,
            search: $searchStore.substring(0, maxSearchLength),
            filter: {
                minDuration:
                    $searchFilter.minDuration !== -1 ? $searchFilter.minDuration : $settings.logs.minEncounterDuration,
                bosses: bosses,
                classes: Array.from($searchFilter.classes),
                cleared: $searchFilter.cleared,
                favorite: $searchFilter.favorite,
                difficulty: $searchFilter.difficulty,
                bossOnlyDamage: $searchFilter.bossOnlyDamage,
            }
        });
        encounters = overview.encounters;
        totalEncounters = overview.totalEncounters;
        NProgress.done();
        return encounters;
    }

    async function refresh() {
        $searchStore = "";
        $pageStore = 1;
        $backNavStore = false;
        await loadEncounters();
    }

    async function nextPage() {
        if ($pageStore * rowsPerPage < totalEncounters) {
            $pageStore++;
            await loadEncounters();
            scrollToTopOfTable();
        }
    }

    async function previousPage() {
        if ($pageStore > 1) {
            $pageStore--;
            await loadEncounters();
            scrollToTopOfTable();
        }
    }

    async function firstPage() {
        $pageStore = 1;
        await loadEncounters();
        scrollToTopOfTable();
    }

    async function lastPage() {
        $pageStore = Math.ceil(totalEncounters / rowsPerPage);
        await loadEncounters();
        scrollToTopOfTable();
    }

    function scrollToTopOfTable() {
        if (encounters.length === 0) {
            return;
        }
        var rows = document.querySelectorAll(`#encounter-${encounters[0].id}`);
        rows[0].scrollIntoView({
            behavior: "smooth",
            block: "center"
        });
    }

    let hidden: boolean = true;
</script>

<svelte:window on:contextmenu|preventDefault />
<LogSidebar bind:hidden />
<div class="h-screen bg-zinc-800">
    <div class="flex h-16 items-center justify-between px-8 py-5 shadow-md">
        <div class="ml-2 flex space-x-2">
            <div class="">
                <button on:click={() => (hidden = false)} class="mt-px block">
                    <svg
                        class="hover:fill-accent-500 h-6 w-6 fill-gray-300"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960"
                        ><path
                            d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z" /></svg>
                </button>
            </div>
            <div class="pl-2 text-xl font-medium tracking-tight text-gray-300">Past Encounters</div>
        </div>
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
                <thead class="sticky top-0 bg-zinc-900 text-xs uppercase">
                    <tr>
                        <th scope="col" class="w-14 px-3 py-3"> ID </th>
                        <th scope="col" class="w-[25%] px-3 py-3"> Encounter </th>
                        <th scope="col" class="px-3 py-3"> Classes </th>
                        <th scope="col" class="hidden w-32 lg:w-48 px-3 py-3 md:table-cell"> Local Player </th>
                        <th scope="col" class="w-14 px-3 py-3"> Dur </th>
                        <th scope="col" class="w-[15%] px-3 py-3 text-right"> Date </th>
                    </tr>
                </thead>
                <tbody class="bg-neutral-800 tracking-tight">
                    {#each encounters as encounter (encounter.fightStart)}
                        <tr class="border-b border-gray-700 hover:bg-zinc-700" id="encounter-{encounter.id}">
                            <td class="px-2 py-3">
                                {#if selectMode}
                                    <div>
                                        <input
                                            type="checkbox"
                                            class="text-accent-500 h-5 w-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0"
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
                                    href="/logs/encounter/?id={encounter.id}"
                                    class="hover:text-accent-500 group flex items-center hover:underline"
                                    use:tooltip={{ content: encounter.bossName }}>
                                    {#if encounter.favorite}
                                        <svg
                                            class="mr-1 h-5 w-5 flex-shrink-0 fill-yellow-400"
                                            xmlns="http://www.w3.org/2000/svg"
                                            viewBox="0 -960 960 960"
                                            ><path
                                                d="m235-82.5 64.5-279.093L83-549l286-25 111-263 111.5 263L877-549 660.484-361.593 725.436-82.5 480.218-230.61 235-82.5Z" /></svg>
                                    {/if}
                                    <div class="truncate">
                                        {#if encounter.difficulty && $settings.general.showDifficulty}
                                            <DifficultyLabel difficulty={encounter.difficulty} hover={true} />
                                            {encounter.bossName}
                                        {:else}
                                            {encounter.bossName}
                                        {/if}
                                    </div>
                                </a>
                            </td>
                            <td class="flex truncate px-3 py-3">
                                {#each encounter.classes as classId, i}
                                    <img
                                        src={$classIconCache[classId]}
                                        alt="class-{classId}"
                                        class="h-8 w-8"
                                        use:tooltip={{ content: encounter.names[i] }} />
                                {/each}
                            </td>
                            <td class="hidden truncate px-3 py-3 md:table-cell">
                                {encounter.localPlayer}
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
            <div class="flex items-center justify-between py-4">
                <span class="text-sm text-gray-400"
                    >Showing <span class="font-semibold dark:text-white"
                        >{($pageStore - 1) * rowsPerPage + 1}-{Math.min(
                            ($pageStore - 1) * rowsPerPage + 1 + rowsPerPage - 1,
                            totalEncounters
                        )}</span>
                    of
                    <span class="font-semibold text-white">{totalEncounters == 0 ? 1 : totalEncounters}</span></span>
                <ul class="inline-flex items-center -space-x-px">
                    <li use:tooltip={{ content: "First" }}>
                        <button class="ml-0 block px-3" on:click={() => firstPage()}>
                            <span class="sr-only">First</span>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960"
                                ><path
                                    d="M226 837V314.5h91.5V837H226Zm459.5-3.5L431 579l254.5-254.5 65.5 65L561.5 579 751 768.5l-65.5 65Z" /></svg
                            ></button>
                    </li>
                    <li use:tooltip={{ content: "Previous" }}>
                        <button class="ml-0 block px-3" on:click={() => previousPage()}>
                            <span class="sr-only">Back</span>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960"
                                ><path d="m560.5 837-262-262 262-262 65 65.5L429 575l196.5 196.5-65 65.5Z" /></svg
                            ></button>
                    </li>
                    <li use:tooltip={{ content: "Next" }}>
                        <button class="ml-0 block px-3" on:click={() => nextPage()}>
                            <span class="sr-only">Next</span>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960"
                                ><path d="m375.5 837-65-65.5L507 575 310.5 378.5l65-65.5 262 262-262 262Z" /></svg
                            ></button>
                    </li>
                    <li use:tooltip={{ content: "Last" }}>
                        <button class="ml-0 block px-3" on:click={() => lastPage()}>
                            <span class="sr-only">Last</span>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-gray-400"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 96 960 960"
                                ><path
                                    d="m273.5 831.5-65.5-65 191-191-191-191 65.5-65 256 256-256 256ZM643 837V314.5h91.5V837H643Z" /></svg
                            ></button>
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
