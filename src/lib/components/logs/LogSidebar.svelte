<script>
    import { getVersion } from '@tauri-apps/api/app';
    import { Drawer } from 'flowbite-svelte';
    import { sineIn } from 'svelte/easing';

    export let hidden = true;

    let transitionParams = {
        x: -320,
        duration: 200,
        easing: sineIn
    };
</script>

<Drawer width="w-52" bgColor="bg-zinc-900" bgOpacity="bg-opacity-50" divClass="bg-zinc-800 z-50" transitionType="fly" {transitionParams} bind:hidden={hidden}>
    <div class="flex justify-between items-center py-4">
        <div class="text-lg font-semibold uppercase text-gray-200 px-4">
            LOA Logs
        </div>
        <button on:click={() => (hidden = true)} class="px-4">
            <svg class="fill-gray-200 h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z"/></svg>
        </button>
    </div>
    <div class="flex flex-col justify-between" style="height: calc(100vh - 3.75rem);">
        <div class="flex flex-col px-4 space-y-4 text-gray-200 pt-4 border-t-2 border-zinc-700">
            <a href="/logs" class="hover:text-accent-500" on:click={() => (hidden = true)}>
                Encounter Logs
            </a>
            <a href="/about" class="hover:text-accent-500" on:click={() => (hidden = true)}>
                About
            </a>
            <a href="/settings" class="hover:text-accent-500" on:click={() => (hidden = true)}>
                Settings
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
