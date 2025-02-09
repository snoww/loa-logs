<script lang="ts">
    import LogDamageMeter from "$lib/components/logs/LogDamageMeter.svelte";
    import { formatTimestamp, getBossHpBars } from "$lib/utils/numbers";
    import {
        backNavStore,
        ifaceChangedStore,
        raidGates,
        screenshotAlert,
        screenshotError,
        searchStore
    } from "$lib/utils/stores";
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";
    import Notification from "$lib/components/shared/Notification.svelte";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import DifficultyLabel from "$lib/components/shared/DifficultyLabel.svelte";
    import BossOnlyDamage from "$lib/components/shared/BossOnlyDamage.svelte";
    import type { PageData } from "./$types";

    interface Props {
        data: PageData;
    }

    let { data }: Props = $props();
    let encounter = $derived(data.encounter);
    let fav = $state(false);
    $effect(() => {
        fav = encounter.favorite;
    });
    let raidGate = $derived($raidGates.get(encounter.currentBossName));

    onMount(() => {
        if ($searchStore.length > 0) {
            $backNavStore = true;
        }
    });

    async function toggle_favorite() {
        await invoke("toggle_encounter_favorite", { id: Number(data.id) });
        fav = !fav;
    }

    let bossHpBars: number | undefined = $state();

    $effect(() => {
        if (encounter) {
            let boss = encounter.entities[encounter.currentBossName];
            if (boss) {
                let bossMaxHpBars = getBossHpBars(boss.name, boss.maxHp);
                bossHpBars = Math.ceil((boss.currentHp / boss.maxHp) * bossMaxHpBars);
            }
        }
    });
</script>

<div class="h-screen bg-zinc-800 pb-20">
    <div class="sticky top-0 z-50 flex h-16 w-full items-center bg-zinc-800 px-8 shadow-md">
        <div class="flex items-center justify-between py-4">
            <a href="/logs" class="bg-accent-900 hover:bg-accent-800 inline-flex rounded-md p-2">
                <span class="sr-only">Back</span>
                <svg class="size-5 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"
                    ><path d="M480 903 153 576l327-327.5 65.5 64.5-216 217h478v91.5h-478l216 216L480 903Z" /></svg>
                <span class="mx-1 text-gray-200">Back</span>
            </a>
        </div>
        <div class="flex items-center justify-between" style="width: calc(100vw - 7.5rem);">
            <div class="flex items-center truncate pl-1 text-xl tracking-tighter">
                <button
                    use:tooltip={{ content: `${fav ? "Remove from" : "Add to"} Favorites` }}
                    onclick={toggle_favorite}>
                    {#if fav}
                        <svg class="size-7 fill-yellow-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                            ><path
                                d="m235-82.5 64.5-279.093L83-549l286-25 111-263 111.5 263L877-549 660.484-361.593 725.436-82.5 480.218-230.61 235-82.5Z" /></svg>
                    {:else}
                        <svg class="size-7 fill-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                            ><path
                                d="m321-202.5 159-95 159 96-42.5-180 140-121.5L552-519.5l-72-170-71.505 169.676L224-504l140 121-43 180.5Zm-86 120 64.5-279.093L83-549l286-25 111-263 111.5 263L877-549 660.484-361.593 725.436-82.5 480.218-230.61 235-82.5Zm245-353Z" /></svg>
                    {/if}
                </button>
                <div class="flex items-center space-x-1 truncate pl-1">
                    {#if $settings.general.showDifficulty && encounter.difficulty}
                        <span class:text-lime-400={encounter.cleared} use:tooltip={{ content: "Cleared" }}
                            >#{data.id.toLocaleString()}:
                        </span>
                        {#if encounter.bossOnlyDamage}
                            <BossOnlyDamage width={2} />
                        {/if}
                        <DifficultyLabel difficulty={encounter.difficulty} />
                        {#if $settings.general.showGate && raidGate}
                            <span class="text-sky-200">[{raidGate}]</span>
                        {/if}
                        {#if !encounter.cleared && bossHpBars}
                            <span class="text-gray-400">[Wipe - {bossHpBars}x]</span>
                        {/if}
                        <div class="truncate" use:tooltip={{ content: encounter.currentBossName }}>
                            {encounter.currentBossName}
                        </div>
                    {:else}
                        <span class:text-lime-400={encounter.cleared}>#{data.id.toLocaleString()}: </span>
                        {#if encounter.bossOnlyDamage}
                            <BossOnlyDamage width={2} />
                        {/if}
                        {#if $settings.general.showGate && raidGate}
                            <span class="text-sky-200">[{raidGate}]</span>
                        {/if}
                        {#if !encounter.cleared && bossHpBars}
                            <span class="text-gray-400">[Wipe - {bossHpBars}x]</span>
                        {/if}
                        <div class="truncate" use:tooltip={{ content: encounter.currentBossName }}>
                            {encounter.currentBossName}
                        </div>
                    {/if}
                </div>
            </div>
            <div class="text-right text-base tracking-tight">
                {formatTimestamp(encounter.fightStart)}
            </div>
        </div>
    </div>
    <div class="overflow-auto bg-zinc-800 pb-8 pl-8 pt-2" style="height: calc(100vh - 4rem);" id="log-breakdown">
        <div class="relative inline-block min-w-[calc(100%-4rem)]">
            <div class="pr-8">
                {#key data.id}
                    <LogDamageMeter id={data.id} {encounter} />
                {/key}
            </div>
        </div>
    </div>
    {#if $screenshotAlert}
        <Notification
            bind:showAlert={$screenshotError}
            text={"Screenshot Copied to Clipboard"}
            dismissable={false}
            width="18rem" />
    {/if}
    {#if $screenshotError}
        <Notification bind:showAlert={$screenshotError} text={"Error Taking Screenshot"} width="18rem" isError={true} />
    {/if}
    {#if $ifaceChangedStore}
        <Notification
            bind:showAlert={$ifaceChangedStore}
            text={"Network Interface Changed. Please fully Restart the App."}
            dismissable={false}
            width="18rem"
            isError={true} />
    {/if}
</div>
