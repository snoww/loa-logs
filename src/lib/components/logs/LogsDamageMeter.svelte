<script lang="ts">
    import { MeterState, MeterTab, type Entity, type Encounter } from "$lib/types";
    import { millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import LogsDamageMeterRow from "./LogsDamageMeterRow.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogBuffs from "./LogBuffs.svelte";

    export let encounter: Encounter;

    let entities: Array<Entity> = [];
    let player: Entity | null = null;
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let classIconsCache: { [key: number]: string } = {};
    
    let anyDead: boolean;


    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let playerName = "";


    $: {       
        if (encounter) {
            entities = Object.values(encounter.entities)
                .filter((players) => players.damageStats.damageDealt > 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
            playerDamagePercentages = entities.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);
            anyDead = entities.some(player => player.isDead);
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
        console.log("inspecting player");
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

</script>

<svelte:window on:contextmenu|preventDefault/>
<LogEncounterInfo encounterDuration={millisToMinutesAndSeconds(encounter.duration)} 
                    totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                    dps={encounter.encounterDamageStats.dps}/>
<div class="flex mt-2">
    <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.DAMAGE} class:bg-gray-700={tab != MeterTab.DAMAGE} on:click={() => tab = MeterTab.DAMAGE}>
        Damage
    </button>
    <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.PARTY_BUFFS} class:bg-gray-700={tab != MeterTab.PARTY_BUFFS} on:click={() => tab = MeterTab.PARTY_BUFFS}>
        Party Synergy
    </button>
    <button class="px-2 rounded-sm py-1" class:bg-pink-900={tab == MeterTab.SELF_BUFFS} class:bg-gray-700={tab != MeterTab.SELF_BUFFS} on:click={() => tab = MeterTab.SELF_BUFFS}>
        Self Synergy
    </button>
</div>
<div class="relative top-0 px" id="buff-table">
    <table class="table-fixed w-full relative">
        {#if tab === MeterTab.DAMAGE}
            {#if state === MeterState.PARTY}
            <thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
                <tr class="bg-zinc-900">
                    <th class="text-left px-2 font-normal w-full"></th>
                    <th class="font-normal w-20" class:hidden={!anyDead}>Dead for</th>
                    <th class="font-normal w-14">DMG</th>
                    <th class="font-normal w-14">DPS</th>
                    <th class="font-normal w-14" class:hidden={entities.length == 1}>D%</th>
                    <th class="font-normal w-14">CRIT</th>
                    <th class="font-normal w-14">F.A</th>
                    <th class="font-normal w-14">B.A</th>
                </tr>
            </thead>
            <tbody>
                {#each entities as entity, i (entity.name)}
                <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(entity.name)}>
                    {#await getClassIconPath(entity.classId) then path}
                        <LogsDamageMeterRow entity={entity} 
                                            percentage={playerDamagePercentages[i]} 
                                            icon={path} 
                                            totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                                            anyDead={anyDead} 
                                            end={encounter.lastCombatPacket}/>
                    {/await}
                </tr>
                {/each}
            </tbody>
            {:else if state === MeterState.PLAYER && player !== null}
               <LogPlayerBreakdown player={player} duration={encounter.duration} handleRightClick={handleRightClick}/>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if state === MeterState.PARTY}
                <LogBuffs tab={tab} encounterDamageStats={encounter.encounterDamageStats} players={entities} percentages={playerDamagePercentages} classIconsCache={classIconsCache} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {:else}
                <LogBuffs tab={tab} encounterDamageStats={encounter.encounterDamageStats} players={entities} percentages={playerDamagePercentages} classIconsCache={classIconsCache} focusedPlayer={player} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if state === MeterState.PARTY}
                <LogBuffs tab={tab} encounterDamageStats={encounter.encounterDamageStats} players={entities} percentages={playerDamagePercentages} classIconsCache={classIconsCache} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {:else}
                <LogBuffs tab={tab} encounterDamageStats={encounter.encounterDamageStats} players={entities} percentages={playerDamagePercentages} classIconsCache={classIconsCache} focusedPlayer={player} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {/if}
        {/if}
    </table>
</div>