<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import Title from "$lib/components/shared/Title.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api";
    import { checkAccessToken, LOG_SITE_URL, uploadLog } from "$lib/utils/sync";
    import type { Encounter } from "$lib/types";
    import { syncStore } from "$lib/utils/stores.js";
    import SettingItem from "$lib/components/settings/SettingItem.svelte";

    let hidden: boolean = true;
    let message = "";

    onMount(async () => {
        await check();

        if (!$syncStore.syncing || $syncStore.stop) {
            syncStore.set({ syncing: false, synced: 0, total: 0, message: "", stop: false });
        }
    });

    async function check() {
        $settings.sync.validToken = await checkAccessToken($settings.sync.accessToken);
        if ($settings.sync.validToken) {
            message = "Valid access token.";
        } else {
            message = "Invalid access token, please generate a new one.";
        }
    }

    function syncPastLogs(force = false) {
        if (!$settings.sync.enabled) {
            $syncStore.message = "Sync is not enabled.";
            return;
        }

        if (!$settings.sync.validToken) {
            $syncStore.message = "Check your access token before syncing past logs.";
            return;
        }

        if ($syncStore.syncing) {
            return;
        }

        $syncStore.syncing = true;
        $syncStore.synced = 0;

        (async () => {
            const ids = (await invoke("get_sync_candidates", { forceResync: force })) as number[];
            console.log(ids);
            $syncStore.total = ids.length;

            for (let i = 0; i < ids.length; i++) {
                let id = ids[i];
                const encounter = (await invoke("load_encounter", { id: id.toString() })) as Encounter;
                let upstream = await uploadLog(id, encounter, $settings.sync);
                if (upstream.id) {
                    $syncStore.synced++;
                }
                $syncStore.message = "Processing logs... (" + i + "/" + ids.length + ")";
                if ($syncStore.stop) {
                    break;
                }
            }
            $syncStore.syncing = false;

            if ($syncStore.synced > 0) {
                $syncStore.message = "Uploaded " + $syncStore.synced + " logs.";
            } else {
                $syncStore.message = "No new logs were uploaded.";
            }

            if ($syncStore.stop) {
                $syncStore.synced = 0;
                $syncStore.total = 0;
                $syncStore.stop = false;
            }
        })();
    }
</script>

<svelte:window on:contextmenu|preventDefault />
<LogSidebar bind:hidden />
<div class="custom-scroll h-screen overflow-y-scroll bg-zinc-800 pb-8">
    <div class="sticky top-0 flex h-16 justify-between bg-zinc-800 px-8 py-5 shadow-md">
        <Title text="Uploading" bind:hidden />
    </div>
    <div class="mx-8 my-4 tracking-tight text-gray-200">
        <p class="px-2 text-base">
            Uploading is currently in alpha, progress is being made by migrating from faust's website.
        </p>
        <div class="mt-2 px-2">
            <SettingItem
                name="Sync (logs.snow.xyz)"
                description="Enable log uploads"
                bind:setting={$settings.sync.enabled} />
        </div>
        <div class="mt-4 flex flex-col space-y-2 px-2">
            <p>Access Token</p>
            <input
                type="password"
                bind:value={$settings.sync.accessToken}
                class="focus:border-accent-500 block w-80 rounded-lg border border-gray-600 bg-zinc-700 text-xs text-zinc-300 placeholder-gray-400 focus:ring-0"
                placeholder="paste access token" />
            <div class="flex space-x-1">
                {#if !$settings.sync.validToken}
                    <a
                        href={LOG_SITE_URL + "/profile"}
                        target="_blank"
                        class="mr-0.5 inline w-fit rounded-md bg-zinc-600 px-1.5 py-1 text-xs hover:bg-zinc-700">
                        Get Access Token
                    </a>
                {/if}
                <button
                    on:click={check}
                    class="mr-0.5 inline w-fit rounded-md bg-zinc-600 px-1.5 py-1 text-xs hover:bg-zinc-700">
                    Check
                </button>
            </div>
            <div class="{$settings.sync.validToken ? 'text-green-400' : 'text-red-500'}">
                {message}
            </div>
        </div>
        <div class="mt-4 px-2">
            <SettingItem
                name="Auto Upload"
                description="Automatically logs when cleared"
                bind:setting={$settings.sync.auto} />
        </div>
        <div class="mt-4 px-2">
            <div class="pb-2">Log Visibility Settings (for future uploads)</div>
            <div class="flex flex-col">
                <div class="flex items-center space-x-2">
                    <input
                        class="checked:bg-accent-500"
                        type="radio"
                        id="op1"
                        name="vis"
                        bind:group={$settings.sync.visibility}
                        value="0" />
                    <label for="op1">Show All Names</label>
                </div>
                <div class="flex items-center space-x-2">
                    <input
                        class="checked:bg-accent-500"
                        type="radio"
                        id="op2"
                        name="vis"
                        bind:group={$settings.sync.visibility}
                        value="1" />
                    <label for="op2">Hide Others (only show self)</label>
                </div>
                <div class="flex items-center space-x-2">
                    <input
                        class="checked:bg-accent-500"
                        type="radio"
                        id="op3"
                        name="vis"
                        bind:group={$settings.sync.visibility}
                        value="2" />
                    <label for="op3">Hide All (anonymous upload)</label>
                </div>
            </div>
        </div>
        <div class="mt-4 flex flex-col space-y-2 px-2">
            <div class="flex items-center space-x-2">
                <div>Sync Past Logs:</div>
                {#if !$syncStore.syncing}
                    <button class="rounded-md bg-zinc-600 p-1 hover:bg-zinc-700" on:click={() => {syncPastLogs();}}>Sync</button>
                    <button class="rounded-md bg-zinc-600 p-1 hover:bg-zinc-700" on:click={() => {syncPastLogs(true);}}>Force Re-sync</button>
                {:else}
                    <button class="rounded-md bg-zinc-600 p-1 hover:bg-zinc-700" disabled>Syncing...</button>
                    <button
                        class="rounded-md bg-zinc-600 p-1 hover:bg-zinc-700"
                        on:click={() => {
                            $syncStore.stop = true;
                        }}>Stop</button>
                {/if}
            </div>
            {#if $syncStore.message}
                <p>{$syncStore.message}</p>
            {/if}
        </div>
    </div>
</div>
