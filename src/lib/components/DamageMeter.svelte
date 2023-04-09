<script lang="ts">
    import { MeterState, MeterTab, type Encounter, type EncounterEvent, type Entity } from '$lib/types';
    import { millisToMinutesAndSeconds } from '$lib/utils/numbers';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { onDestroy, onMount } from 'svelte';
    import { flip } from 'svelte/animate';
    import EncounterInfo from './EncounterInfo.svelte';
    import BossInfo from './BossInfo.svelte';
    import DamageMeterPlayerRow from './DamageMeterPlayerRow.svelte';
    import PlayerBreakdown from './PlayerBreakdown.svelte';
    import Footer from './Footer.svelte';
    import Buffs from './Buffs.svelte';
    import { resourceDir } from '@tauri-apps/api/path';
    import { Alert } from 'flowbite-svelte'
    import { fade } from 'svelte/transition';

    let time = +Date.now();
    let encounter: Encounter | null = null;
    let events: Array<UnlistenFn> = [];

    let zoneChangeAlert = false;
    let phaseTransitionAlert = false;
    let raidEndAlert = false;

    onMount(() => {
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        (async () => {
            let encounterUpdateEvent = await listen('encounter-update', (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                encounter = event.payload;
            });
            let zoneChangeEvent = await listen('zone-change', (event) => {
                console.log("zone change event")
                zoneChangeAlert = true;
                setTimeout(() => {
                    state = MeterState.PARTY;
                    player = null;
                    playerName = "";
                    encounter = null;
                    entities = [];
                    currentBoss = null;
                    encounterDuration = "00:00";
                    totalDamageDealt = 0;
                    dps = 0;
                    zoneChangeAlert = false
                }, 6000);
            });
            let phaseTransitionEvent = await listen('phase-transition', (event) => {
                console.log("phase transition event: ", event.payload)
                // phaseTransitionAlert = true;
                // setTimeout(() => {
                //     phaseTransitionAlert = false;
                // }, 3000);
            });
            let raidEndEvent = await listen('raid-end', (event: EncounterEvent) => {
                console.log("raid-end, updating encounter")
                encounter = event.payload;
                raidEndAlert = true;
                setTimeout(() => {
                    raidEndAlert = false;
                }, 3000);
            });

            events.push(
                encounterUpdateEvent, 
                zoneChangeEvent,
                phaseTransitionEvent,
                raidEndEvent
            );
            // encounter = JSON.parse(await readTextFile(await documentDir() + 'projects\\loa-log-parser\\2023-03-11-02-39-58-Demon-Beast-Commander-Valtan.json'));
            // console.log(encounter);
        })();
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    let entities: Array<Entity> = [];
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let encounterDuration = "00:00";
    let duration = 0;
    let totalDamageDealt = 0;
    let dps = 0;
    let currentBoss: Entity | null = null;
    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let player: Entity | null = null;
    let playerName = "";

    $: {
        if (encounter) {
            if (encounter.fightStart !== 0) {
                entities = Object.values(encounter.entities)
                    .filter((players) => players.damageStats.damageDealt > 0)
                    .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
                playerDamagePercentages = entities.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);            
                duration = time - encounter.fightStart;
                if (duration < 0) {
                    encounterDuration = millisToMinutesAndSeconds(0);
                    dps = 0;
                } else {
                    encounterDuration = millisToMinutesAndSeconds(duration);
                    dps = totalDamageDealt / (duration / 1000);
                }
                totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt;
            }
            
            if (encounter.currentBoss !== undefined) {
                currentBoss = encounter.currentBoss;
            }

            if (playerName) {
                player = encounter.entities[playerName];
                state = MeterState.PLAYER;

            } else {
                player = null;
                state = MeterState.PARTY;
            }
        }
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
</script>

<svelte:window on:contextmenu|preventDefault/>
<EncounterInfo encounterDuration={encounterDuration} totalDamageDealt={totalDamageDealt} dps={dps}/>
{#if currentBoss !== null}
<div class="relative top-7">
    <BossInfo boss={currentBoss}/>
</div>
{/if}
{#await resourceDir() then path}
<div class="relative top-7 overflow-scroll" style="height: calc(100vh - 1.5rem - 1.75rem {currentBoss !== null ? " - 1.75rem" : ""});">
    <table class="table-fixed w-full relative">
        {#if tab === MeterTab.DAMAGE}
            {#if state === MeterState.PARTY}
            <thead class="top-0 sticky h-6" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
                <tr class="bg-zinc-900">
                    <th class="text-left px-2 font-normal w-full"></th>
                    <!-- <th class="">DMG</th> -->
                    <th class="font-normal w-14">DPS</th>
                    <th class="font-normal w-14" class:hidden={entities.length == 1}>D%</th>
                    <th class="font-normal w-14">CRIT</th>
                    <th class="font-normal w-14">F.A</th>
                    <th class="font-normal w-14">B.A</th>
                </tr>
            </thead>
            <tbody>
                {#each entities as entity, i (entity.id)}
                <tr class="h-7 px-2 py-1" animate:flip="{{duration: 200}}" on:click={() => inspectPlayer(entity.name)}>
                    <DamageMeterPlayerRow
                        entity={entity}
                        percentage={playerDamagePercentages[i]}
                        duration={duration}
                        totalDamageDealt={totalDamageDealt}
                    />
                </tr>
                {/each}
            </tbody>
            {:else if state === MeterState.PLAYER && player !== null}
                <PlayerBreakdown player={player} duration={duration} handleRightClick={handleRightClick}/>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs tab={tab} encounterDamageStats={encounter?.encounterDamageStats} players={entities} percentages={playerDamagePercentages} path={path} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {:else}
                <Buffs tab={tab} encounterDamageStats={encounter?.encounterDamageStats} players={entities} percentages={playerDamagePercentages} path={path} focusedPlayer={player} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs tab={tab} encounterDamageStats={encounter?.encounterDamageStats} players={entities} percentages={playerDamagePercentages} path={path} focusedPlayer={player} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {:else}
                <Buffs tab={tab} encounterDamageStats={encounter?.encounterDamageStats} players={entities} percentages={playerDamagePercentages} path={path} focusedPlayer={player} handleRightClick={handleRightClick} inspectPlayer={inspectPlayer}/>
            {/if}
        {/if}
    </table>
    {#if zoneChangeAlert}
        <div transition:fade>
            <Alert color="none" class="bg-pink-800 bg-opacity-80 w-48 mx-auto absolute inset-x-0 bottom-1 py-2 z-50" dismissable on:close={() => zoneChangeAlert = false}>
                <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
                </span>
                Changing Zone
            </Alert>
        </div>
    {/if}
    {#if phaseTransitionAlert}
    <div transition:fade>
        <Alert color="none" class="bg-pink-800 bg-opacity-80 w-52 mx-auto absolute inset-x-0 bottom-1 py-2 z-50" dismissable on:close={() => phaseTransitionAlert = false}>
            <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
            </span>
            Phase Transition
        </Alert>
    </div>
    {/if}
    {#if raidEndAlert}
    <div transition:fade>
        <Alert color="none" class="bg-pink-800 bg-opacity-80 w-48 mx-auto absolute inset-x-0 bottom-1 py-2 z-50" dismissable on:close={() => raidEndAlert = false}>
            <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
            </span>
            Raid Ending
        </Alert>
    </div>
    {/if}
</div>
{/await}
<Footer bind:tab={tab}/>