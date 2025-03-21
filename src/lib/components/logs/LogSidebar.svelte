<script lang="ts">
    import { updateSettings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api";
    import { getVersion } from "@tauri-apps/api/app";
    import { checkUpdate } from "@tauri-apps/api/updater";
    import { Drawer } from "flowbite-svelte";
    import { onMount } from "svelte";
    import { sineIn } from "svelte/easing";
    import { writable } from "svelte/store";

    interface Props {
        hidden?: boolean;
    }

    let { hidden = $bindable(true) }: Props = $props();

    let transitionParams = {
        x: -320,
        duration: 200,
        easing: sineIn
    };
    let spin = writable(false);
    let updateText = writable("Check for Updates");

    let loaRunning = $state(false);

    onMount(() => {
        checkLoaRunning();
    });

    async function checkLoaRunning() {
        loaRunning = await invoke("check_loa_running");
    }

    function startLoa() {
        invoke("start_loa_process");
    }
    let starting = $state(false);

    $effect(() => {
        if (!hidden) {
            checkLoaRunning();

            setInterval(() => {
                checkLoaRunning();
            }, 5000);
        }
    });
</script>

<Drawer
    width="w-52"
    bgColor="bg-zinc-900"
    bgOpacity="opacity-50"
    divClass="bg-zinc-800 z-50"
    transitionType="fly"
    {transitionParams}
    bind:hidden>
    <div class="flex items-center justify-between py-4">
        <div class="px-4 text-lg font-semibold text-gray-200 uppercase">LOA Logs</div>
        <button onclick={() => (hidden = true)} class="px-4" aria-label="Close">
            <svg class="size-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960">
                <path
                    d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" />
            </svg>
        </button>
    </div>
    <div class="flex flex-col justify-between" style="height: calc(100vh - 3.75rem);">
        <div class="flex flex-col space-y-4 border-t-2 border-zinc-700 px-4 pt-4 text-gray-200">
            <a href="/logs" class="hover:text-accent-500" onclick={() => (hidden = true)}> Encounter Logs </a>
            <a href="/upload" class="hover:text-accent-500" onclick={() => (hidden = true)}> Uploading </a>
            <a href="/about" class="hover:text-accent-500" onclick={() => (hidden = true)}> About </a>
            <a href="/settings" class="hover:text-accent-500" onclick={() => (hidden = true)}> Settings </a>
            <a href="/changelog" class="hover:text-accent-500" onclick={() => (hidden = true)}> Changelog </a>
            <a
                href="https://ko-fi.com/synow"
                class="hover:text-accent-500"
                target="_blank"
                onclick={() => (hidden = true)}>
                <div class="inline-flex items-center space-x-1">
                    <div>Donate</div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-2 fill-gray-300" viewBox="0 0 512 512">
                        <path
                            d="M432,320H400a16,16,0,0,0-16,16V448H64V128H208a16,16,0,0,0,16-16V80a16,16,0,0,0-16-16H48A48,48,0,0,0,0,112V464a48,48,0,0,0,48,48H400a48,48,0,0,0,48-48V336A16,16,0,0,0,432,320ZM488,0h-128c-21.37,0-32.05,25.91-17,41l35.73,35.73L135,320.37a24,24,0,0,0,0,34L157.67,377a24,24,0,0,0,34,0L435.28,133.32,471,169c15,15,41,4.5,41-17V24A24,24,0,0,0,488,0Z" />
                    </svg>
                </div>
            </a>
            {#if !loaRunning && !starting}
                <button
                    class="bg-accent-800 hover:bg-accent-900 mx-4 rounded-lg p-2"
                    onclick={() => {
                        starting = true;
                        startLoa();
                        setTimeout(() => {
                            starting = false;
                            checkLoaRunning();
                        }, 20000);
                    }}>
                    Start Lost Ark
                </button>
            {:else if !loaRunning && starting}
                <button class="mx-4 rounded-lg bg-zinc-700 p-2" disabled> Starting... </button>
            {:else if loaRunning}
                <button class="mx-4 rounded-lg bg-zinc-700 p-2" disabled> Lost Ark Running </button>
            {/if}
        </div>
        <div class="px-3 py-2 text-gray-300">
            <div class="flex items-center justify-between">
                <div>
                    {#await getVersion()}
                        version
                    {:then version}
                        version {version}
                    {/await}
                </div>
                {#if $updateSettings.available}
                    <button
                        class="pr-1"
                        aria-label="Update Now"
                        use:tooltip={{ content: "Update Now" }}
                        onclick={() => {
                            $updateSettings.dismissed = false;
                        }}>
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 -960 960 960"
                            class="fill-accent-500 size-5 animate-bounce">
                            <path
                                d="M281.5-165v-57.5H679v57.5H281.5Zm170-165v-356L329-563.5 289-604l191-191 191.5 191-40.5 40.5L509-686v356h-57.5Z" />
                        </svg>
                    </button>
                {:else}
                    <button
                        class="pr-1"
                        aria-label="Check for Updates"
                        use:tooltip={{ content: $updateText }}
                        onclick={async () => {
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
                                    $updateSettings.dismissed = false;
                                    $updateSettings.available = true;
                                    $updateSettings.manifest = manifest;
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
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 -960 960 960"
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
