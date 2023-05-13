<script lang="ts">
    import { goto } from "$app/navigation";
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import TableFilter from "$lib/components/table/TableFilter.svelte";
    import type { EncounterPreview, EncountersOverview } from "$lib/types";
    import { formatDurationFromMs, formatTimestamp } from "$lib/utils/numbers";
    import { classIconCache, settings } from "$lib/utils/settings";
    import { backNavStore, pageStore, searchStore } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api";
    import NProgress from 'nprogress';
    import 'nprogress/nprogress.css';

    let encounters: Array<EncounterPreview> = [];
    let totalEncounters: number = 0;
    const rowsPerPage = 10;
    const maxSearchLength = 30;

    let oldDbExists = false;

    $: {       
        if ($searchStore.length > 0) {
            if ($backNavStore) {
                $backNavStore = false;
            } else {
                $pageStore = 1;
            }
        }

        loadEncounters();
        checkOldDbExists();
    }

    async function loadEncounters(): Promise<Array<EncounterPreview>> {        
        let overview: EncountersOverview = await invoke("load_encounters_preview", { page: $pageStore, pageSize: rowsPerPage, minDuration: $settings.logs.minEncounterDuration, search: $searchStore.substring(0, maxSearchLength) });        
        encounters = overview.encounters;
        totalEncounters = overview.totalEncounters;
        return encounters;
    }

    async function refresh() {
        $searchStore = "";
        $pageStore = 1;
        $backNavStore = false;
        NProgress.start();
        let promise = loadEncounters();
        await promise;
        NProgress.done();
    }

    async function nextPage() {
        if ($pageStore * rowsPerPage < totalEncounters) {
            $pageStore++;
            await loadEncounters();
            scrollToTopOfTable()
        }
    }

    async function previousPage() {
        if ($pageStore > 1) {
            $pageStore--;
            await loadEncounters();
            scrollToTopOfTable()
        }
    }

    async function firstPage() {
        $pageStore = 1;
        await loadEncounters();
        scrollToTopOfTable()
    }

    async function lastPage() {
        $pageStore = Math.ceil(totalEncounters / rowsPerPage);
        await loadEncounters();
        scrollToTopOfTable()
    }

    function scrollToTopOfTable() {
        if (encounters.length === 0) {
            return;
        }
        var rows = document.querySelectorAll(`#encounter-${encounters[0].id}`);
        rows[0].scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
    }

    let hidden: boolean = true;

    async function checkOldDbExists() {
        oldDbExists = await invoke("check_old_db_location_exists");
    }

    async function getEncounterCount(): Promise<number> {       
        return await invoke("get_encounter_count");
    }

    async function copyDb() {
        NProgress.start();
        await invoke("copy_db");
        NProgress.done();
        setTimeout(async () => {
            await refresh()
        }, 2000);
    }

    async function openFolder(path: string) {
        await invoke("open_folder", { path: path });
    }

</script>

<svelte:window on:contextmenu|preventDefault/>
<LogSidebar bind:hidden={hidden}/>
<div class="bg-zinc-800 h-screen">
        <div class="px-8 flex justify-between shadow-md py-5 h-16 items-center">
            <div class="flex space-x-2 ml-2">
                <div class="">
                    <button on:click={() => (hidden = false)} class="block mt-px">
                        <svg class="fill-gray-300 w-6 h-6 hover:fill-accent-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z"/></svg>
                    </button>
                </div>
                <div class="text-xl font-bold tracking-tight text-gray-300 pl-2">
                    Past Encounters
                </div>
            </div>
            <button class="px-2 py-1 rounded-md bg-accent-900 hover:bg-accent-800 mr-4 shadow-md" on:click={() => refresh()}>
                Refresh
            </button>
        </div>
    <div class="px-8">
        <div class="py-2">
            <TableFilter/>
        </div>
        <div class="relative overflow-x-hidden overflow-y-auto" style="height: calc(100vh - 8.25rem - 2.5rem);" id="logs-table">
            <table class="w-full text-left text-gray-400 table-fixed" id="table">
                <thead class="text-xs uppercase bg-zinc-900 top-0 sticky">
                    <tr>
                        <th scope="col" class="px-3 py-3 w-[7%]">
                            ID
                        </th>
                        <th scope="col" class="px-3 py-3 w-[30%]">
                            Encounter
                        </th>
                        <th scope="col" class="px-3 py-3">
                            Classes
                        </th>
                        <th scope="col" class="px-3 py-3 w-[12%]">
                            Duration
                        </th>
                    </tr>
                </thead>
                <tbody class="tracking-tight bg-neutral-800">
                    {#each encounters as encounter (encounter.fightStart)}
                        <tr class="border-b border-gray-700" id="encounter-{encounter.id}">
                            <td class="px-2 py-3">
                                <div use:tooltip={{content: formatTimestamp(encounter.fightStart)}}>
                                    #{encounter.id}
                                </div>
                            </td>
                            <td class="px-3 py-3 font-bold text-gray-300 w-full truncate">
                                <a href="/logs/encounter/?id={encounter.id}" class="hover:underline hover:text-accent-500">
                                    {encounter.bossName}
                                </a>
                            </td>
                            <td class="px-3 py-3">
                                {#each encounter.classes as classId, i }
                                    <img src={$classIconCache[classId]} alt="class-{classId}" class="w-8 h-8 inline-block" use:tooltip={{content: encounter.names[i]}}/>
                                {/each}
                            </td>
                            <td class="px-3 py-3 text-center">
                                {formatDurationFromMs(encounter.duration)}
                            </td>
                        </tr>
                    {:else}
                        {#if $searchStore.length > 0}
                            <div class="w-screen bg-neutral-800 p-2">No encounters found.</div>
                        {:else}
                        <div class="w-screen bg-neutral-800 p-2">No encounters recorded.</div>
                        <div class="w-screen bg-neutral-800 p-2">Meter should be turned on at character select (before entering raid at latest) for best accuracy.</div>
                        {/if}
                        {#await getEncounterCount() then count}
                        {#if count === 0 && oldDbExists}
                            <div class="w-screen h-px bg-gray-700"></div>
                            <div class="w-screen bg-neutral-800 p-2 font-bold text-lg">!!! NOTICE !!!</div>
                            <div class="w-screen bg-neutral-800 px-2">The default install directory of the app has changed after v1.1.12</div>
                            <div class="w-screen bg-neutral-800 px-2">If you had logs previously, you must manually copy the <span class="font-mono">encounters.db</span> to the new install location. :( sorry!</div>
                            <!-- svelte-ignore a11y-click-events-have-key-events -->
                            <div class="w-screen bg-neutral-800 px-2">Old Default: <span class="font-mono text-white hover:underline cursor-pointer" on:click={() => openFolder("USERPROFILE\\AppData\\Local\\Programs\\LOA Logs")}>C:\Users\USERNAME\AppData\Local\Programs\LOA Logs</span></div>
                            <!-- svelte-ignore a11y-click-events-have-key-events -->
                            <div class="w-screen bg-neutral-800 px-2">New Default: <span class="font-mono text-white hover:underline cursor-pointer" on:click={() => openFolder("USERPROFILE\\AppData\\Local\\LOA Logs")}>C:\Users\USERNAME\AppData\Local\LOA Logs</span></div>
                            <div class="w-screen bg-neutral-800 p-2"><button class="p-2 rounded-md bg-accent-900 text-white hover:bg-accent-800" on:click={copyDb}>I'm too lazy. Help me copy please.</button></div>
                            {/if}
                        {/await}
                    {/each}
                </tbody>
            </table>
        </div>
        {#if encounters.length > 0}
        <div class="flex items-center justify-between py-4">
            <span class="text-sm text-gray-400">Showing <span class="font-semibold dark:text-white">{($pageStore - 1) * rowsPerPage + 1}-{Math.min(($pageStore - 1) * rowsPerPage + 1 + rowsPerPage - 1, totalEncounters)}</span> of <span class="font-semibold text-white">{totalEncounters == 0 ? 1 : totalEncounters}</span></span>
            <ul class="inline-flex items-center -space-x-px">
                <li use:tooltip={{content: 'First'}}>
                    <button class="block px-3 ml-0" on:click={() => firstPage()}>
                        <span class="sr-only">First</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-accent-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M226 837V314.5h91.5V837H226Zm459.5-3.5L431 579l254.5-254.5 65.5 65L561.5 579 751 768.5l-65.5 65Z"/></svg></button>
                </li>
                <li use:tooltip={{content: 'Previous'}}>
                    <button class="block px-3 ml-0" on:click={() => previousPage()}>
                        <span class="sr-only">Back</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-accent-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m560.5 837-262-262 262-262 65 65.5L429 575l196.5 196.5-65 65.5Z"/></svg></button>
                </li>
                <li use:tooltip={{content: 'Next'}}>
                    <button class="block px-3 ml-0" on:click={() => nextPage()}>
                        <span class="sr-only">Next</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-accent-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m375.5 837-65-65.5L507 575 310.5 378.5l65-65.5 262 262-262 262Z"/></svg></button>
                </li>
                <li use:tooltip={{content: 'Last'}}>
                    <button class="block px-3 ml-0" on:click={() => lastPage()}>
                        <span class="sr-only">Last</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-accent-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m273.5 831.5-65.5-65 191-191-191-191 65.5-65 256 256-256 256ZM643 837V314.5h91.5V837H643Z"/></svg></button>
                </li>
            </ul>
        </div>
        {/if}
    </div>
</div>