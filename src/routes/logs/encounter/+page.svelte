<script lang="ts">
    import { page } from '$app/stores';
    import LogDamageMeter from '$lib/components/logs/LogDamageMeter.svelte';
    import type { Encounter } from '$lib/types';
    import { formatTimestamp } from '$lib/utils/numbers';
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount } from 'svelte';

    let encounter: Encounter;
    let currentPage: number = 1;
    let id: string;

    onMount(() => {        
        if ($page.url.searchParams.has('page')) {
            currentPage = parseInt($page.url.searchParams.get('page')!);
        }
    })

    async function loadEncounter() {
        id = $page.url.searchParams.get('id')!;
        encounter = await invoke("load_encounter", { id: id });
    }
</script>

<div class="bg-zinc-800 h-screen overflow-y-scroll pb-20 pt-4" id="log-breakdown">
    {#await loadEncounter() then _}
    <div class="px-8 flex items-center">
        <div class="flex items-center justify-between py-4">
            <a href="/logs?page={currentPage}" class="p-2 rounded-md bg-pink-900 hover:bg-pink-800 inline-flex">
                <span class="sr-only">Back</span>
                <svg class="w-5 h-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M480 903 153 576l327-327.5 65.5 64.5-216 217h478v91.5h-478l216 216L480 903Z"/></svg>
                <span class="mx-1 text-gray-200">Back</span>
            </a>
        </div>
        <div class="flex justify-between w-full items-center">
            <div class="text-xl font-bold tracking-tight text-gray-300 pl-2">
                #{(id).toLocaleString()}: {encounter.currentBossName}
            </div>
            <div class="text-base">
                {formatTimestamp(encounter.fightStart)}
            </div>
        </div>
    </div>
    <div class="px-8">
        <LogDamageMeter id={id} encounter={encounter} />
    </div>
    {/await}
</div>