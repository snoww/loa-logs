<script lang="ts">
    import { page } from "$app/stores";
    import LogDamageMeter from "$lib/components/logs/LogDamageMeter.svelte";
    import type { Encounter } from "$lib/types";
    import { formatTimestamp } from "$lib/utils/numbers";
    import {
        backNavStore,
        ifaceChangedStore,
        screenshotAlert,
        screenshotError,
        searchStore
    } from "$lib/utils/stores";
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";
    import Notification from "$lib/components/shared/Notification.svelte";

    let id: string;
    let promise: Promise<Encounter>;

    onMount(() => {
        if ($searchStore.length > 0) {
            $backNavStore = true;
        }
    });

    $: {
        id = $page.url.searchParams.get("id")!;
        promise = invoke("load_encounter", { id: id });
    }
</script>

<div class="h-screen bg-zinc-800 pb-20">
    {#await promise then encounter}
        <div class="sticky top-0 z-50 flex h-16 w-full items-center bg-zinc-800 px-8 shadow-md">
            <div class="flex items-center justify-between py-4">
                <a href="/logs" class="bg-accent-900 hover:bg-accent-800 inline-flex rounded-md p-2">
                    <span class="sr-only">Back</span>
                    <svg class="h-5 w-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"
                        ><path d="M480 903 153 576l327-327.5 65.5 64.5-216 217h478v91.5h-478l216 216L480 903Z" /></svg>
                    <span class="mx-1 text-gray-200">Back</span>
                </a>
            </div>
            <div class="flex w-full items-center justify-between">
                <div class="truncate pl-2 text-xl font-bold tracking-tight text-gray-300">
                    #{id.toLocaleString()}: {encounter.currentBossName}
                </div>
                <div class="text-base">
                    {formatTimestamp(encounter.fightStart)}
                </div>
            </div>
        </div>
        <div class="overflow-auto bg-zinc-800 pb-8 pl-8 pt-2" style="height: calc(100vh - 4rem);" id="log-breakdown">
            <div class="relative inline-block min-w-[calc(100%-4rem)]">
                <div class="pr-8">
                    <LogDamageMeter {id} {encounter} />
                </div>
            </div>
        </div>
    {/await}
    {#if $screenshotAlert}
    <Notification bind:showAlert={$screenshotError} text={"Screenshot Copied to Clipboard"} width="20rem"/>
    {/if}
    {#if $screenshotError}
    <Notification bind:showAlert={$screenshotError} text={"Error Taking Screenshot"} width="18rem" isError={true}/>
    {/if}
    {#if $ifaceChangedStore}
    <Notification bind:showAlert={$ifaceChangedStore} text={"Network Interface Changed. Please fully Restart the App."} dismissable={false} width="18rem" isError={true}/>
    {/if}
</div>
