<script lang="ts">
    import { MeterState, MeterTab, type Entity, type Encounter } from "$lib/types";
    import { millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
    import LogsDamageMeterRow from "./LogsDamageMeterRow.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogBuffs from "./LogBuffs.svelte";
    import { page } from "$app/stores";
    import { writable } from "svelte/store";
    import { hideNames } from "$lib/utils/stores";

    export let id: string;
    export let encounter: Encounter;

    let players: Array<Entity> = [];
    let player: Entity | null = null;
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let classIconsCache: { [key: number]: string } = {};
    
    let anyDead: boolean;


    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let playerName = "";

    let deleteConfirm = false;

    $: {       
        if (encounter) {
            players = Object.values(encounter.entities)
                .filter((players) => players.damageStats.damageDealt > 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
            playerDamagePercentages = players.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);
            anyDead = players.some(player => player.isDead);
        }
        
        if (playerName) {
            player = encounter.entities[playerName];
            state = MeterState.PLAYER;

        } else {
            player = null;
            state = MeterState.PARTY;
        }        
    }

    async function getClassIconPath(classId: number) {       
        if (classId in classIconsCache) {
            return classIconsCache[classId];
        }
        let path;
        if (classId > 100) {
            path = `${classId}.png`;
        } else {
            path = `${1}/101.png`;
        }
        let resolvedPath = convertFileSrc(await join(await resourceDir(), 'images', 'classes', path));
        classIconsCache[classId] = resolvedPath;
        return resolvedPath;
    }

    function inspectPlayer(name: string) {
        state = MeterState.PLAYER;
        playerName = name;
    }

    function handleRightClick() {
        if (state === MeterState.PLAYER) {
            state = MeterState.PARTY;
            player = null;
            playerName = "";
        }
    }

    async function deleteEncounter() {
        await invoke("delete_encounter", { id: id });
        if ($page.url.searchParams.has('page')) {
            let currentPage = parseInt($page.url.searchParams.get('page')!);
            document.location.href = `/logs?page=${currentPage}`;
        } else {
            document.location.href = "/logs";
        }
    }

</script>

<svelte:window on:contextmenu|preventDefault/>
<LogEncounterInfo encounterDuration={millisToMinutesAndSeconds(encounter.duration)} 
                    totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                    dps={encounter.encounterDamageStats.dps}/>
<div class="mt-2 flex justify-between">
    <div class="flex divide-x divide-gray-600">
        <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.DAMAGE} class:bg-gray-700={tab != MeterTab.DAMAGE} on:click={() => tab = MeterTab.DAMAGE}>
            Damage
        </button>
        <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.PARTY_BUFFS} class:bg-gray-700={tab != MeterTab.PARTY_BUFFS} on:click={() => tab = MeterTab.PARTY_BUFFS}>
            Party Synergy
        </button>
        <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.SELF_BUFFS} class:bg-gray-700={tab != MeterTab.SELF_BUFFS} on:click={() => tab = MeterTab.SELF_BUFFS}>
            Self Synergy
        </button>
        <div class="flex items-center px-2 space-x-2 bg-gray-700 rounded">
            <span class="text-sm font-medium">Hide Names</span>
            <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" value="" class="sr-only peer" on:click={() => hideNames.update(h => !h)}>
                <div class="w-9 h-5 peer-focus:outline-none peer-focus:ring-pink-800 rounded-full peer bg-gray-800 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all border-gray-600 peer-checked:bg-pink-800"></div>
              </label>
        </div>
    </div>
    <button class="bg-red-900 hover:bg-red-800 rounded-md px-2 mb-1 shadow-md" on:click={() => deleteConfirm = true}>
        Delete
    </button>
    {#if deleteConfirm}
    <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80"></div>
    <div class="fixed top-0 left-0 right-0 h-modal z-50 w-full p-4 justify-center items-center">
        <div class="flex relative max-w-md w-full max-h-full">
            <div class="bg-zinc-800 text-gray-400 rounded-lg border-gray-700 shadow-md relative flex flex-col mx-auto">
                <button type="button" class="focus:outline-none whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 ml-auto absolute top-3 right-2.5" aria-label="Close modal" on:click={() => deleteConfirm = false}>
                    <span class="sr-only">Close modal</span> <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"></path></svg>
                </button> 
                <div id="modal" class="p-6 space-y-6 flex-1 overflow-y-auto overscroll-contain">
                    <div class="text-center">
                        <svg aria-hidden="true" class="mx-auto mb-4 w-14 h-14 text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" class="s-Qbr4I8QhaoSZ"></path></svg> 
                        <h3 class="mb-5 text-lg font-normal text-gray-400">Are you sure you want to delete this encounter?</h3> 
                        <button type="button" class="text-center font-medium focus:outline-none inline-flex items-center justify-center px-5 py-2.5 text-sm text-white bg-red-700 hover:bg-red-800 rounded-lg mr-2" on:click={deleteEncounter}>
                            Yes, I'm sure
                        </button> 
                        <button type="button" class="text-center font-medium focus:outline-none inline-flex items-center justify-center px-5 py-2.5 text-sm bg-gray-800 text-gray-400 focus:text-white hover:text-white hover:bg-zinc-700 bg-transparent rounded-lg" on:click={() => deleteConfirm = false}>
                            No, cancel
                        </button>
                    </div>
                </div> 
            </div>
        </div>
    </div>
    {/if}
</div>
<div class="relative top-0 px" id="buff-table">
    <table class="table-fixed w-full relative">
        {#if tab === MeterTab.DAMAGE}
            {#if state === MeterState.PARTY}
            <thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
                <tr class="bg-zinc-900">
                    <th class="text-left px-2 font-normal w-full"></th>
                    {#if anyDead}
                    <th class="font-normal w-20">Dead for</th>
                    {/if}
                    <th class="font-normal w-14">DMG</th>
                    <th class="font-normal w-14">DPS</th>
                    {#if players.length > 1}
                    <th class="font-normal w-14">D%</th>
                    {/if}
                    <th class="font-normal w-14">CRIT</th>
                    <th class="font-normal w-14">F.A</th>
                    <th class="font-normal w-14">B.A</th>
                </tr>
            </thead>
            <tbody>
                {#each players as player, i (player.name)}
                <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
                    {#await getClassIconPath(player.classId) then path}
                        <LogsDamageMeterRow entity={player} 
                                            percentage={playerDamagePercentages[i]} 
                                            icon={path} 
                                            totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                                            {anyDead} 
                                            end={encounter.lastCombatPacket}
                                           />
                    {/await}
                </tr>
                {/each}
            </tbody>
            {:else if state === MeterState.PLAYER && player !== null}
               <LogPlayerBreakdown {player} duration={encounter.duration} {handleRightClick}/>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if state === MeterState.PARTY}
                <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {classIconsCache} {handleRightClick} {inspectPlayer}/>
            {:else}
                <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {classIconsCache} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if state === MeterState.PARTY}
                <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {classIconsCache} {handleRightClick} {inspectPlayer}/>
            {:else}
                <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {classIconsCache} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
            {/if}
        {/if}
    </table>
</div>