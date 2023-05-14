<script lang="ts">
    import { MeterState, MeterTab, type Encounter, type EncounterEvent, type Entity, EntityType } from '$lib/types';
    import { millisToMinutesAndSeconds } from '$lib/utils/numbers';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { onDestroy, onMount } from 'svelte';
    import { flip } from 'svelte/animate';
    import EncounterInfo from './EncounterInfo.svelte';
    import BossInfo from './BossInfo.svelte';
    import DamageMeterPlayerRow from './DamageMeterRow.svelte';
    import PlayerBreakdown from './PlayerBreakdown.svelte';
    import Footer from './Footer.svelte';
    import Buffs from './Buffs.svelte';
    import { Alert } from 'flowbite-svelte'
    import { fade } from 'svelte/transition';
    import { settings } from '$lib/utils/settings';
    import { tooltip } from '$lib/utils/tooltip';

    let time = +Date.now();
    let encounter: Encounter | null = null;
    let events: Array<UnlistenFn> = [];

    let zoneChangeAlert = false;
    let resettingAlert = false;
    let pauseAlert = false;
    let phaseTransitionAlert = false;
    let bossDeadAlert = false;
    let raidInProgress = true;
    let adminAlert = false;

    onMount(() => {
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        (async () => {
            let encounterUpdateEvent = await listen('encounter-update', (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                encounter = event.payload;
            });
            let zoneChangeEvent = await listen('zone-change', (event: any) => {
                // console.log("zone change event")
                zoneChangeAlert = true;
                setTimeout(() => {
                    reset();
                    zoneChangeAlert = false;
                }, 6000);
            });
            let raidStartEvent = await listen('raid-start', (event: any) => {
                reset();
                raidInProgress = true;
            });
            let resetEncounterEvent = await listen('reset-encounter', (event: any) => {                
                reset();
                resettingAlert = true;
                setTimeout(() => {
                    resettingAlert = false;
                }, 1500);
            });
            let pauseEncounterEvent = await listen('pause-encounter', (event: any) => {
                paused = !paused;
                pauseAlert = !pauseAlert;
            });
            let phaseTransitionEvent = await listen('phase-transition', (event: any) => {
                let phaseCode = event.payload;
                // console.log(Date.now() + ": phase transition event: ", event.payload)
                if (phaseCode === 1) {
                    bossDeadAlert = true;
                    setTimeout(() => {
                        bossDeadAlert = false;
                    }, 3000);
                } else if (phaseCode === 2 && raidInProgress) {
                    phaseTransitionAlert = true;
                    setTimeout(() => {
                        phaseTransitionAlert = false;
                    }, 3000);
                }
                raidInProgress = false;
            });
            let adminError = await listen('admin', (event: any) => {
                adminAlert = true;
            });

            events.push(
                encounterUpdateEvent, 
                zoneChangeEvent,
                resetEncounterEvent,
                pauseEncounterEvent,
                phaseTransitionEvent,
                raidStartEvent,
                adminError
            );
        })();
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    let players: Array<Entity> = [];
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
    let lastCombatPacket = 0;
    let anyDead: boolean = false;
    let anyFrontAtk: boolean = false;
    let anyBackAtk: boolean = false;
    let anySupportBuff: boolean = false;
    let anySupportBrand: boolean = false;

    let paused = false;

    $: {
        if (encounter) {                 
            if (encounter.fightStart !== 0 && raidInProgress && !paused) {
                if ($settings.general.showEsther) {
                    players = Object.values(encounter.entities)
                        .filter((e) => e.damageStats.damageDealt > 0 && (e.entityType === EntityType.ESTHER || e.entityType === EntityType.PLAYER))
                        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                } else {
                    players = Object.values(encounter.entities)
                        .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER)
                        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                }
                anyDead = players.some(player => player.isDead);
                anyFrontAtk = players.some(player => player.skillStats.frontAttacks > 0);
                anyBackAtk = players.some(player => player.skillStats.backAttacks > 0);
                anySupportBuff = players.some(player => player.damageStats.buffedBySupport > 0);
                anySupportBrand = players.some(player => player.damageStats.debuffedBySupport > 0);
                topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
                playerDamagePercentages = players.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);
                
                if (encounter.currentBoss && !encounter.currentBoss.isDead || !encounter.currentBoss) {
                    duration = time - encounter.fightStart;
                }
                
                if (duration < 0) {
                    encounterDuration = millisToMinutesAndSeconds(0);
                    dps = 0;
                } else {
                    encounterDuration = millisToMinutesAndSeconds(duration);
                    dps = totalDamageDealt / (duration / 1000);
                }
                if ($settings.general.showEsther) {
                    totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt
                        + players
                            .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.ESTHER)
                            .reduce((a, b) => a + b.damageStats.damageDealt, 0);
                } else {
                    totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt;
                }

                lastCombatPacket = encounter.lastCombatPacket;
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

    function reset() {
        state = MeterState.PARTY;
        player = null;
        playerName = "";
        encounter = null;
        players = [];
        currentBoss = null;
        encounterDuration = "00:00";
        totalDamageDealt = 0;
        dps = 0;
        anyDead = false;
        anyFrontAtk = false;
        anyBackAtk = false;
        anySupportBuff = false;
        anySupportBrand = false;
    }
</script>

<svelte:window on:contextmenu|preventDefault/>
<EncounterInfo {encounterDuration} {totalDamageDealt} {dps}/>
{#if currentBoss !== null && $settings.meter.bossHp}
<div class="relative top-7">
    <BossInfo boss={currentBoss}/>
</div>
{/if}
<div class="relative top-7 overflow-scroll" style="height: calc(100vh - 1.5rem - 1.75rem {currentBoss !== null ? " - 1.75rem" : ""});">
    <table class="table-fixed w-full relative">
        {#if tab === MeterTab.DAMAGE}
            {#if state === MeterState.PARTY}
            <thead class="top-0 sticky h-6 z-40" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
                <tr class="bg-zinc-900 tracking-tighter">
                    <th class="w-7 px-2 font-normal"></th>
                    <th class="text-left px-2 font-normal w-14"></th>
                    <th class="w-full"></th>
                    {#if anyDead && $settings.meter.deathTime}
                    <th class="font-normal w-14" use:tooltip={{content: "Dead for"}}>Dead</th>
                    {/if}
                    {#if $settings.meter.damage}
                    <th class="font-normal w-14" use:tooltip={{content: "Damage Dealt"}}>DMG</th>
                    {/if}
                    {#if $settings.meter.dps}
                    <th class="font-normal w-14" use:tooltip={{content: "Damage per second"}}>DPS</th>
                    {/if}
                    {#if players.length > 1 && $settings.meter.damagePercent}
                    <th class="font-normal w-12" use:tooltip={{content: "Damage %"}}>D%</th>
                    {/if}
                    {#if $settings.meter.critRate}
                    <th class="font-normal w-12" use:tooltip={{content: "Crit %"}}>CRIT</th>
                    {/if}
                    {#if anyFrontAtk && $settings.meter.frontAtk}
                    <th class="font-normal w-12" use:tooltip={{content: "Front Attack %"}}>F.A</th>
                    {/if}
                    {#if anyBackAtk && $settings.meter.backAtk}
                    <th class="font-normal w-12" use:tooltip={{content: "Back Attack %"}}>B.A</th>
                    {/if}
                    {#if anySupportBuff && $settings.meter.percentBuffBySup}
                    <th class="font-normal w-12" use:tooltip={{content: "% Damage buffed by Support"}}>Buff%</th>
                    {/if}
                    {#if anySupportBrand && $settings.meter.percentBrand}
                    <th class="font-normal w-12" use:tooltip={{content: "% Damage buffed by Brand"}}>B%</th>
                    {/if}
                    {#if $settings.meter.counters}
                    <th class="font-normal w-12" use:tooltip={{content: "Counters"}}>CTR</th>
                    {/if}
                </tr>
            </thead>
            <tbody>
                {#each players as entity, i (entity.id)}
                <tr class="h-7 px-2 py-1" animate:flip="{{duration: 200}}" on:click={() => inspectPlayer(entity.name)}>
                    <DamageMeterPlayerRow
                        {entity}
                        percentage={playerDamagePercentages[i]}
                        {duration}
                        {totalDamageDealt}
                        {lastCombatPacket}
                        {anyDead}
                        {anyFrontAtk}
                        {anyBackAtk}
                        {anySupportBuff}
                        {anySupportBrand}
                    />
                </tr>
                {/each}
            </tbody>
            {:else if state === MeterState.PLAYER && player !== null}
                <PlayerBreakdown entity={player} duration={duration} {handleRightClick}/>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs {tab} encounterDamageStats={encounter?.encounterDamageStats} {players} percentages={playerDamagePercentages} {handleRightClick} {inspectPlayer}/>
            {:else}
                <Buffs {tab} encounterDamageStats={encounter?.encounterDamageStats} {players} percentages={playerDamagePercentages} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs {tab} encounterDamageStats={encounter?.encounterDamageStats} {players} percentages={playerDamagePercentages} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
            {:else}
                <Buffs {tab} encounterDamageStats={encounter?.encounterDamageStats} {players} percentages={playerDamagePercentages} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
            {/if}
        {/if}
    </table>
</div>
{#if zoneChangeAlert}
<div transition:fade>
    <Alert color="none" class="bg-accent-800 bg-opacity-80 w-48 mx-auto absolute inset-x-0 bottom-8 py-2 z-50" dismissable on:close={() => zoneChangeAlert = false}>
        <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
        </span>
        Changing Zone
    </Alert>
</div>
{/if}
{#if resettingAlert}
<div transition:fade>
    <Alert color="none" class="bg-accent-800 bg-opacity-80 w-40 mx-auto absolute inset-x-0 bottom-8 py-2 z-50" dismissable on:close={() => resettingAlert = false}>
        <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
        </span>
        Resetting
    </Alert>
</div>
{/if}
{#if pauseAlert}
<div transition:fade>
    <Alert color="none" class="bg-accent-800 bg-opacity-80 w-32 mx-auto absolute inset-x-0 bottom-8 py-2 z-50" on:close={() => pauseAlert = false}>
        <span slot="icon">
            <svg class="w-5 h-5" fill="currentColor" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M555 852V300h172.5v552H555Zm-322 0V300h172.5v552H233Z"/></svg>
        </span>
        Paused
    </Alert>
</div>
{/if}
{#if phaseTransitionAlert}
<div transition:fade>
    <Alert color="none" class="bg-accent-800 bg-opacity-80 w-52 mx-auto absolute inset-x-0 bottom-8 py-2 z-50" dismissable on:close={() => phaseTransitionAlert = false}>
        <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
        </span>
        Wipe/Phase Clear
    </Alert>
</div>
{/if}
{#if bossDeadAlert}
<div transition:fade>
    <Alert color="none" class="bg-accent-800 bg-opacity-80 w-48 mx-auto absolute inset-x-0 bottom-8 py-2 z-50" dismissable on:close={() => bossDeadAlert = false}>
        <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
        </span>
        Boss Dead
    </Alert>
</div>
{/if}
{#if adminAlert}
<div transition:fade>
    <Alert color="none" class="bg-red-700 w-56 mx-auto absolute inset-x-0 bottom-8 py-2 z-50">
        <span slot="icon"><svg aria-hidden="true" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>
        </span>
        Please restart as Admin
    </Alert>
</div>
{/if}
<Footer bind:tab={tab}/>