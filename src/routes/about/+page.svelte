<script lang="ts">
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import {
        backNavStore,
        pageStore,
        searchStore
    } from "$lib/utils/stores";
    import { getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";
    import Title from "$lib/components/shared/Title.svelte";

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
<div class="custom-scroll h-screen overflow-y-scroll bg-zinc-800 pb-8">
    <div class="sticky top-0 flex h-16 justify-between bg-zinc-800 px-8 py-5 shadow-md">
        <Title text="About" bind:hidden />
    </div>
    <div class="mx-8 my-4 tracking-tight text-gray-200">
        <p class="px-4 text-base">
            LOA Logs is a "blazingly fast" open source Lost Ark DPS meter (<a
            class="text-accent-500 hover:underline"
            href="https://github.com/snoww/loa-logs"
            target="_blank">snoww/loa-logs</a
        >), written in Rust by
            <a class="text-accent-500 hover:underline" href="https://github.com/snoww" target="_blank">Snow</a>. This
            project is an opinionated flavor of
            <a
                class="text-accent-500 hover:underline"
                href="https://github.com/lost-ark-dev/loa-details"
                target="_blank">LOA Details</a>
            by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing and
            processing
            has been completely ported over to Rust, with
            <a class="text-accent-500 hover:underline" href="https://github.com/snoww/meter-core-rs" target="_blank"
            ><code>meter-core-rs</code></a
            >. A huge thanks to Herysia and Henjuro for their work on the original
            <a class="text-accent-500 hover:underline" href="https://github.com/lost-ark-dev/meter-core" target="_blank"
            ><code>meter-core</code></a
            >. This gives the meter huge performance improvements with low memory usage compared the TypeScript
            implementation.
        </p>
        <p class="mt-4 px-4 text-base">This project was designed specifically for hell-raiding.</p>
        <p class="mt-4 px-4 text-base">
            If you have any problems or suggestions, please open an <a
            class="text-accent-500 hover:underline"
            href="https://github.com/snoww/loa-logs/issues"
            target="_blank">issue</a>
            or send a message in the <code>#loa-logs</code> channel on Discord at
            <a class="text-accent-500 hover:underline" href="https://discord.gg/sbSa3pkDF5" target="_blank"
            ><code>discord.gg/sbSa3pkDF5</code></a>
        </p>

        <p class="mt-4 px-4 text-base">You can support me and this project by buying me a coffee.</p>
        <a href="https://www.buymeacoffee.com/synow" target="_blank"
        ><img
            class="mt-2 px-4"
            src="/bmac.png"
            alt="Buy Me A Coffee" /></a>

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
