<script>
    import { getVersion } from "@tauri-apps/api/app";
    import { Drawer } from "flowbite-svelte";
    import { sineIn } from "svelte/easing";
    import { tooltip } from "$lib/utils/tooltip";
    import { updateAvailable, updateDismissed, updateManifest } from "$lib/utils/stores";
    import { invoke } from "@tauri-apps/api";
    import { checkUpdate } from "@tauri-apps/api/updater";
    import { writable } from "svelte/store";

    export let hidden = true;

    let transitionParams = {
        x: -320,
        duration: 200,
        easing: sineIn
    };
    let spin = writable(false);
    let updateText = writable("Check for Updates");
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
            <svg class="size-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"
            >
                <path
                    d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" />
            </svg>
        </button>
    </div>
    <div class="flex flex-col justify-between" style="height: calc(100vh - 3.75rem);">
        <div class="flex flex-col space-y-4 border-t-2 border-zinc-700 px-4 pt-4 text-gray-200">
            <a href="/logs" class="hover:text-accent-500" on:click={() => (hidden = true)}> Encounter Logs </a>
            <a href="/about" class="hover:text-accent-500" on:click={() => (hidden = true)}> About </a>
            <a href="/settings" class="hover:text-accent-500" on:click={() => (hidden = true)}> Settings </a>
            <a href="https://www.buymeacoffee.com/synow" class="hover:text-accent-500" target="_blank"
               on:click={() => (hidden = true)}>
                <div class="inline-flex space-x-1 items-center">
                    <div>Donate</div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-2 fill-gray-300" viewBox="0 0 512 512">
                        <path
                            d="M432,320H400a16,16,0,0,0-16,16V448H64V128H208a16,16,0,0,0,16-16V80a16,16,0,0,0-16-16H48A48,48,0,0,0,0,112V464a48,48,0,0,0,48,48H400a48,48,0,0,0,48-48V336A16,16,0,0,0,432,320ZM488,0h-128c-21.37,0-32.05,25.91-17,41l35.73,35.73L135,320.37a24,24,0,0,0,0,34L157.67,377a24,24,0,0,0,34,0L435.28,133.32,471,169c15,15,41,4.5,41-17V24A24,24,0,0,0,488,0Z" />
                    </svg>
                </div>
            </a>
        </div>
        <div class="px-3 py-2 text-gray-300">
            <div class="flex justify-between items-center">
                <div>
                    {#await getVersion()}
                        version
                    {:then version}
                        version {version}
                    {/await}
                </div>
                {#if $updateAvailable}
                    <button class="pr-1" use:tooltip={{content: "Update Now"}}
                            on:click={() => {$updateDismissed = false;}}>
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                             class="size-5 fill-accent-500 animate-bounce">
                            <path
                                d="M281.5-165v-57.5H679v57.5H281.5Zm170-165v-356L329-563.5 289-604l191-191 191.5 191-40.5 40.5L509-686v356h-57.5Z" />
                        </svg>
                    </button>
                {:else}
                    <button class="pr-1" use:tooltip={{content: $updateText}} on:click={async () => {
                        if ($updateText === "No Updates Available") return;
                        if (!$spin) {
                            $spin = true;
                            setTimeout(() => {
                                $spin = false;
                            }, 1000);
                        }
                        try {
                            const { shouldUpdate, manifest } = await checkUpdate();
                            if (shouldUpdate) {
                                $updateDismissed = false;
                                $updateAvailable = true;
                                $updateManifest = manifest;
                            } else {
                                $updateText = "No Updates Available";
                                setTimeout(() => {
                                    $updateText = "Check for Updates";
                                }, 5000);
                            }
                        } catch (e) {
                            await invoke("write_log", { message: e });
                        }
                    }}>
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                             class="size-5 fill-gray-300 {$spin ? 'animate-spin-once' : ''}">
                            <path
                                d="M169.333-164.667V-228h123.334l-16.666-14.666q-58.167-49.834-84.834-108.317Q164.5-409.467 164.5-477.598q0-105.735 62.48-189.332t163.686-114.403v65.999Q316.866-687.258 272.35-622q-44.517 65.257-44.517 144.213 0 56.787 21.083 101.954 21.084 45.167 59.751 79.834L334-276.666v-116h63.333v227.999h-228ZM570-178v-66.666q74.167-28 118.334-93.241 44.166-65.241 44.166-144.64 0-46.453-21.25-93.62t-58.583-84.5L628-683.334v116.001h-63.333v-228h228V-732H668.666l16.667 16q55.899 53.062 83.2 114.186 27.3 61.124 27.3 119.314 0 105.833-62.333 189.833T570-178Z" />
                        </svg>
                    </button>
                {/if}
            </div>
        </div>
    </div>
</Drawer>