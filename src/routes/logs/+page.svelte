<script lang="ts">
    import type { Encounter } from "$lib/types";
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";

    let encounters: Array<Encounter> = [];

    async function loadEncounters(): Promise<Array<Encounter>> {
        encounters = await invoke("load_encounters");
        return encounters;
    }

    $: {
        if (encounters.length > 0) {
            console.log(encounters);
        }
    }

</script>
<div class="bg-zinc-800 h-screen">
    <div class="px-10 pt-5">
        <div class="text-xl">
            Past Encounters
        </div>
        <div class="pt-5">
            {#await loadEncounters() then encounters}
            {#each encounters as encounter (encounter.fightStart)}
                <div class="text-white">
                    {encounter.currentBossName}
                </div>
            {:else}
                <div>
                    No past encounters
                </div>
            {/each}
            {/await}
        </div>
    </div>
</div>