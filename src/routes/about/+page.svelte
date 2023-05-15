<script lang="ts">
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import { backNavStore, pageStore, searchStore } from "$lib/utils/stores";
    import { getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";

    let hidden: boolean = true;

    onMount(() => {
        // dunno if this is good lol XD
        $pageStore = 1;
        $backNavStore = false;
        $searchStore = "";
    });
</script>

<svelte:window on:contextmenu|preventDefault />
<LogSidebar bind:hidden />
<div class="h-screen bg-zinc-800 pt-2">
    <div class="px-8 pt-5 tracking-tight">
        <div class="flex justify-between">
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
                <div class="pl-2 text-xl font-bold tracking-tight text-gray-300">About</div>
            </div>
        </div>
        <p class="mt-12 px-4 text-base">
            LOA Logs is a "blazingly fast" open source Lost Ark DPS meter (<a
                class="text-accent-500 hover:underline"
                href="https://github.com/snoww/loa-logs"
                target="_blank">github.com/snoww/loa-logs</a
            >), written in Rust by
            <a class="text-accent-500 hover:underline" href="https://github.com/snoww" target="_blank">Snow</a>. This
            project is an opinionated flavor of
            <a
                class="text-accent-500 hover:underline"
                href="https://github.com/lost-ark-dev/loa-details"
                target="_blank">LOA Details</a>
            by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing is still
            done by LOA Details'
            <a class="text-accent-500 hover:underline" href="https://github.com/lost-ark-dev/meter-core" target="_blank"
                ><code>meter-core</code></a> under the hood, but the data processing is done using Rust. There are future
            plans to port the packet sniffing part to Rust as well.
        </p>
        <p class="mt-4 px-4 text-base">This project was designed specifically for hell-raiding.</p>
        <p class="mt-4 px-4 text-base">
            If you have any problems or suggestions, please open an <a
                class="text-accent-500 hover:underline"
                href="https://github.com/snoww/loa-logs/issues"
                target="_blank">issue</a> or send a DM on Discord to Snow#7777.
        </p>

        <p class="mt-4 px-4">
            Current version:
            {#await getVersion()}
                v
            {:then version}
                v{version}
            {/await}
        </p>
    </div>
</div>
