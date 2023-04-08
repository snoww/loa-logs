<script lang="ts">
import LogSidebar from '$lib/components/logs/LogSidebar.svelte';
import type { PageData } from '../about copy/$types';
    
    export let data: PageData;

    let hidden: boolean = true;

</script>

<LogSidebar bind:hidden={hidden} />
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
                    Settings
                </div>
            </div>
        </div>
    </div>
</div>