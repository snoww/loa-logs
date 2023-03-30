<script lang="ts">
    import type { Encounter, EncounterEvent, Entity } from '$lib/types';
    import { millisToMinutesAndSeconds } from '$lib/utils/numbers';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { onDestroy, onMount } from 'svelte';
    import DamageMeterPlayer from './DamageMeterPlayer.svelte';
    import { flip } from 'svelte/animate';
    import EncounterInfo from './EncounterInfo.svelte';
    import BossInfo from './BossInfo.svelte';
    import DamageMeterPlayerRow from './DamageMeterPlayerRow.svelte';

    $: time = +Date.now();
    let encounterEvent: Encounter;
    $: encounter = encounterEvent;
    
    let rustEventUnlisten: UnlistenFn;

    onMount(() => {
        console.log('the component has mounted');
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        (async () => {
            rustEventUnlisten = await listen('rust-event', (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                // console.log(event.payload.currentBoss);
                encounterEvent = event.payload;
                // loaLog = Date.now() + " " + event.payload;
            });
        })();
    });

    onDestroy(() => {
        console.log('the component has unmounted, wow');
        if (rustEventUnlisten) {
            rustEventUnlisten();
        }
    });

    let entities: Array<Entity> = [];
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let encounterDuration = "00:00";
    let duration = 0;
    let totalDamageDealt = 0;
    let dps = 0;
    let currentBoss: Entity | null = null;

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
        }
    }
</script>

<EncounterInfo encounterDuration={encounterDuration} totalDamageDealt={totalDamageDealt} dps={dps}/>
{#if currentBoss !== null}
    <BossInfo boss={currentBoss}/>
{/if}
<table class="border-collapse table-auto w-full">
    <thead>
        <tr class="bg-zinc-900">
            <th class="text-left px-2 font-normal">Name</th>
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
        <tr class="h-8 px-2 py-1" animate:flip="{{duration: 200}}">
            <DamageMeterPlayerRow
                entity={entity}
                percentage={playerDamagePercentages[i]}
                duration={duration}
                totalDamageDealt={totalDamageDealt}
            />
        </tr>
        {/each}
    </tbody>
</table>