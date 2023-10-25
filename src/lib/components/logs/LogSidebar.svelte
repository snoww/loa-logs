<script>
    import { getVersion } from "@tauri-apps/api/app";
    import { Drawer } from "flowbite-svelte";
    import { sineIn } from "svelte/easing";

    export let hidden = true;

    let transitionParams = {
        x: -320,
        duration: 200,
        easing: sineIn
    };
</script>

<Drawer
    width="w-52"
    bgColor="bg-zinc-900"
    bgOpacity="bg-opacity-50"
    divClass="bg-zinc-800 z-50"
    transitionType="fly"
    {transitionParams}
    bind:hidden>
    <div class="flex items-center justify-between py-4">
        <div class="px-4 text-lg font-semibold uppercase text-gray-200">LOA Logs</div>
        <button on:click={() => (hidden = true)} class="px-4">
            <svg class="h-5 w-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"
                ><path
                    d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" /></svg>
        </button>
    </div>
    <div class="flex flex-col justify-between" style="height: calc(100vh - 3.75rem);">
        <div class="flex flex-col space-y-4 border-t-2 border-zinc-700 px-4 pt-4 text-gray-200">
            <a href="/logs" class="hover:text-accent-500" on:click={() => (hidden = true)}> Encounter Logs </a>
            <a href="/about" class="hover:text-accent-500" on:click={() => (hidden = true)}> About </a>
            <a href="/settings" class="hover:text-accent-500" on:click={() => (hidden = true)}> Settings </a>
            <a href="https://www.buymeacoffee.com/synow" class="hover:text-accent-500" target="_blank" on:click={() => (hidden = true)}>
                <div class="inline-flex space-x-1 items-center">
                    <div>Donate</div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-2 fill-gray-300" viewBox="0 0 512 512"><path d="M432,320H400a16,16,0,0,0-16,16V448H64V128H208a16,16,0,0,0,16-16V80a16,16,0,0,0-16-16H48A48,48,0,0,0,0,112V464a48,48,0,0,0,48,48H400a48,48,0,0,0,48-48V336A16,16,0,0,0,432,320ZM488,0h-128c-21.37,0-32.05,25.91-17,41l35.73,35.73L135,320.37a24,24,0,0,0,0,34L157.67,377a24,24,0,0,0,34,0L435.28,133.32,471,169c15,15,41,4.5,41-17V24A24,24,0,0,0,488,0Z"/></svg>
                </div>
            </a>
        </div>
        <div class="px-3 py-2 text-gray-300">
            {#await getVersion()}
                version
            {:then version}
                version {version}
            {/await}
        </div>
    </div>
</Drawer>
