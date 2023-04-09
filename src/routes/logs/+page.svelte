<script lang="ts">
    import { page } from "$app/stores";
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import type { EncounterPreview, EncountersOverview } from "$lib/types";
    import { formatDurationFromMs, formatTimestamp } from "$lib/utils/numbers";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
    import { Tooltip } from 'flowbite-svelte';
    import { onMount } from "svelte";


    let encounters: Array<EncounterPreview> = [];
    let totalEncounters: number = 0;
    let currentPage = 1;
    const rowsPerPage = 10;
    let classIconsCache: { [key: number]: string } = {}

    async function loadEncounters(page: number = 1): Promise<Array<EncounterPreview>> {
        if ($page.url.searchParams.has('page')) {
            page = parseInt($page.url.searchParams.get('page')!);
            $page.url.searchParams.delete('page');
        }
        let overview: EncountersOverview = await invoke("load_encounters_preview", { page: page, pageSize: rowsPerPage });
        encounters = overview.encounters;
        totalEncounters = overview.totalEncounters;
        currentPage = page;
        return encounters;
    }

    async function getClassIconPath(classId: number) {
        if (classId in classIconsCache) {
            return classIconsCache[classId];
        }
        let path;
        if (classId > 100) {
            path = `${classId}.png`;
        } else {
            path = `${1}/101.png`;
        }
        let resolvedPath = convertFileSrc(await join(await resourceDir(), 'images', 'classes', path));
        classIconsCache[classId] = resolvedPath;
        return resolvedPath;
    }

    async function refresh() {
        await loadEncounters();
        scrollToTopOfTable()
    }

    async function nextPage() {
        if (currentPage * rowsPerPage < totalEncounters) {
            currentPage++;
            await loadEncounters(currentPage);
            scrollToTopOfTable()
        }
    }

    async function previousPage() {
        if (currentPage > 1) {
            currentPage--;
            await loadEncounters(currentPage);
            scrollToTopOfTable()
        }
    }

    async function firstPage() {
        currentPage = 1;
        await loadEncounters(currentPage);
        scrollToTopOfTable()
    }

    async function lastPage() {
        currentPage = Math.ceil(totalEncounters / rowsPerPage);
        await loadEncounters(currentPage);
        scrollToTopOfTable()
    }

    function scrollToTopOfTable() {
        var rows = document.querySelectorAll('#table tr');
        rows[0].scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
    }

    let hidden: boolean = true;

</script>

<svelte:window on:contextmenu|preventDefault/>
<LogSidebar bind:hidden={hidden}/>
<div class="bg-zinc-800 h-screen pt-2">
    <div class="px-8 pt-5">
        <div class="flex justify-between">
            <div class="flex space-x-2 ml-2">
                <div class="">
                    <button on:click={() => (hidden = false)} class="block mt-px">
                        <svg class="fill-gray-300 w-6 h-6 hover:fill-pink-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z"/></svg>
                    </button>
                </div>
                <div class="text-xl font-bold tracking-tight text-gray-300 pl-2">
                    Past Encounters
                </div>
            </div>
            <button class="px-2 py-1 rounded-md bg-pink-900 hover:bg-pink-800 mr-4 shadow-md" on:click={() => refresh()}>
                Refresh
            </button>
        </div>
        <div class="mt-4 relative overflow-x-hidden overflow-y-scroll" style="height: calc(100vh - 8.25rem);" id="logs-table">
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
                    {#await loadEncounters() then _}
                    {#each encounters as encounter (encounter.fightStart)}
                        <tr class="border-b border-gray-700">
                            <td class="px-3 py-3">
                                <div>
                                    #{encounter.id}
                                </div>
                                <Tooltip defaultClass="bg-pink-800 p-2 text-gray-300">{formatTimestamp(encounter.fightStart)}</Tooltip>
                            </td>
                            <td class="px-3 py-3 font-bold text-gray-300 w-full truncate">
                                <a href="/logs/encounter/?id={encounter.id}&page={currentPage}" class="hover:underline">
                                    {encounter.bossName}
                                </a>
                            </td>
                            <td class="px-3 py-3">
                                {#each encounter.classes as classId }
                                    {#await getClassIconPath(classId) then path }
                                        <img src={path} alt="class-{classId}" class="w-8 h-8 inline-block" />
                                    {/await}
                                {/each}
                            </td>
                            <td class="px-3 py-3 text-center">
                                {formatDurationFromMs(encounter.duration)}
                            </td>
                        </tr>
                    {:else}
                        <div class="w-screen bg-neutral-800 p-2">No encounters recorded so far.</div>
                        <div class="w-screen bg-neutral-800 p-2">Meter should be turned on at character select for best accuracy.</div>
                    {/each}
                    {/await}
                </tbody>
            </table>
        </div>
        {#if encounters.length > 0}
        <div class="flex items-center justify-between py-4">
            <span class="text-sm text-gray-400">Showing <span class="font-semibold dark:text-white">{(currentPage - 1) * rowsPerPage + 1}-{Math.min((currentPage - 1) * rowsPerPage + 1 + rowsPerPage - 1, totalEncounters)}</span> of <span class="font-semibold text-white">{totalEncounters}</span></span>
            <ul class="inline-flex items-center -space-x-px">
                <li>
                    <button class="block px-3 ml-0" on:click={() => firstPage()}>
                        <span class="sr-only">First</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-pink-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M226 837V314.5h91.5V837H226Zm459.5-3.5L431 579l254.5-254.5 65.5 65L561.5 579 751 768.5l-65.5 65Z"/></svg></button>
                </li>
                <li>
                    <button class="block px-3 ml-0" on:click={() => previousPage()}>
                        <span class="sr-only">Back</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-pink-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m560.5 837-262-262 262-262 65 65.5L429 575l196.5 196.5-65 65.5Z"/></svg></button>
                </li>
                <li>
                    <button class="block px-3 ml-0" on:click={() => nextPage()}>
                        <span class="sr-only">Next</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-pink-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m375.5 837-65-65.5L507 575 310.5 378.5l65-65.5 262 262-262 262Z"/></svg></button>
                </li>
                <li>
                    <button class="block px-3 ml-0" on:click={() => lastPage()}>
                        <span class="sr-only">Last</span>
                        <svg class="w-5 h-5 fill-gray-400 hover:fill-pink-800" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m273.5 831.5-65.5-65 191-191-191-191 65.5-65 256 256-256 256ZM643 837V314.5h91.5V837H643Z"/></svg></button>
                </li>
            </ul>
        </div>
        {/if}
    </div>
</div>