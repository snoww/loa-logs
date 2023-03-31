<script lang="ts">
    import { MeterState, type Encounter, type EncounterEvent, type Entity } from '$lib/types';
    import { millisToMinutesAndSeconds } from '$lib/utils/numbers';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { onDestroy, onMount } from 'svelte';
    import { flip } from 'svelte/animate';
    import EncounterInfo from './EncounterInfo.svelte';
    import BossInfo from './BossInfo.svelte';
    import DamageMeterPlayerRow from './DamageMeterPlayerRow.svelte';
    import PlayerBreakdown from './PlayerBreakdown.svelte';

    let time = +Date.now();
    let encounter: Encounter | null = null;
    let events: Array<UnlistenFn> = [];

    onMount(() => {
        console.log('the component has mounted');
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        (async () => {
            let encounterUpdateEvent = await listen('encounter-update', (event: EncounterEvent) => {
                console.log(+Date.now(), event.payload);
                // console.log(event.payload.currentBoss);
                encounter = event.payload;
                // loaLog = Date.now() + " " + event.payload;
            });
            let zoneChangeEvent = await listen('zone-change', (event) => {
                console.log("zone change event")
                state = MeterState.LIVE;
                player = null;
                playerName = "";
                encounter = null;
                entities = [];
                encounterDuration = "00:00";
                totalDamageDealt = 0;
                dps = 0;
            });
            let phaseTransitionEvent = await listen('phase-transition', (event) => {
                console.log("phase transition event")
            });

            events.push(
                encounterUpdateEvent, 
                zoneChangeEvent,
                phaseTransitionEvent
            );
            // encounter = JSON.parse(await readTextFile(await documentDir() + 'projects\\loa-log-parser\\2023-03-11-02-39-58-Demon-Beast-Commander-Valtan.json'));
            // console.log(encounter);
        })();
    });

    onDestroy(() => {
        console.log('the component has unmounted, wow');
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
    let state = MeterState.LIVE;
    let player: Entity | null = null;
    let playerName = "";

    $: {
        if (encounter) {
            // if (encounter.reset) {
            //     setTimeout(() => {}, 5000);
            // }
            if (encounter.fightStart !== 0) {
                entities = Object.values(encounter.entities)
                    .filter((players) => players.damageStats.damageDealt > 0)
                    .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
                playerDamagePercentages = entities.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);            
                if (!encounter.reset) {
                    duration = time - encounter.fightStart;
                }
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
                state = MeterState.LIVE;
            }
        }
    }

    function inspectPlayer(name: string) {
        console.log("inspecting player");
        state = MeterState.PLAYER;
        playerName = name;
    }

    function handleRightClick() {
        if (state === MeterState.PLAYER) {
            state = MeterState.LIVE;
            player = null;
            playerName = "";
        }
    }
</script>

<svelte:window on:contextmenu|preventDefault={handleRightClick}/>
<EncounterInfo encounterDuration={encounterDuration} totalDamageDealt={totalDamageDealt} dps={dps}/>
{#if currentBoss !== null}
<div class="relative top-7">
    <BossInfo boss={currentBoss}/>
</div>
{/if}
<div class="relative top-7 overflow-y-scroll" style="height: calc(100vh - 1.7rem);">
    <table class="table-fixed w-full">
        {#if state === MeterState.LIVE}
        <thead class="top-0 sticky">
            <tr class="bg-zinc-900">
                <th class="text-left px-2 font-normal w-full">Name</th>
                <!-- <th class="">DMG</th> -->
                <th class="font-normal w-14">DPS</th>
                <th class="font-normal w-14">D%</th>
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
            <PlayerBreakdown player={player} duration={duration}/>
        {/if}
    </table>
</div>