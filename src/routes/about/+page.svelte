<script lang="ts">
import LogSidebar from '$lib/components/logs/LogSidebar.svelte';
import { getVersion } from '@tauri-apps/api/app';
import type { PageData } from './$types';
    
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
                    About
                </div>
            </div>
        </div>
        <p class="px-4 mt-4 text-base">
            LOA Logs is a "blazingly fast" open source Lost Ark DPS meter (<a class="text-pink-400 hover:underline" href="https://github.com/snoww/loa-logs" target="_blank">Github</a>),
            written in Rust by <a class="text-pink-400 hover:underline" href="https://github.com/snoww" target="_blank">Snow</a>. 
            This project is a very opinionated flavor of <a class="text-pink-400 hover:underline" href="https://github.com/lost-ark-dev/loa-details" target="_blank">LOA Details</a> by Herysia and Mathi, but should share very similar user interfaces and settings. 
            The packet sniffing is still done by LOA Details' <a class="text-pink-400 hover:underline" href="https://github.com/lost-ark-dev/meter-core" target="_blank"><code>meter-core</code></a> under the hood, 
            but the data processing is done using Rust. There are future plans to port the packet sniffing part to Rust as well.
        </p>
        <p class="px-4 mt-4 text-base">
            Features: Stylized Boss HP Bar, Simple UI, Fast logging performance, Log sharing, Small log sizes, Encounter filtering, DPS charts, Fight replay, and more.
        </p>
        <p class="px-4 mt-4 text-base">
            If you have any problems or suggestions, please open an <a class="text-pink-400 hover:underline" href="https://github.com/snoww/loa-logs/issues" target="_blank">issue</a> or send a DM on Discord.
        </p>

        <p class="px-4 mt-4">
            Current version: 
            {#await getVersion()}
            v
            {:then version}
            v{version}
            {/await}
        </p>
    </div>
</div>