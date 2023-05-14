<script lang="ts">
    import { classesMap } from "$lib/constants/classes";
    import { StatusEffectTarget, type EncounterDamageStats, type Entity, MeterTab, type StatusEffect, EntityType } from "$lib/types";
    import { defaultBuffFilter } from "$lib/utils/buffs";
    import LogBuffHeader from "./LogBuffHeader.svelte";
    import LogBuffRow from "./LogBuffRow.svelte";
    import LogBuffBreakdown from "./LogBuffBreakdown.svelte";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats;
    export let players: Array<Entity>;
    export let focusedPlayer: Entity | null = null;
    export let handleRightClick: () => void;
    export let inspectPlayer: (name: string) => void;

    if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
        focusedPlayer = null;
        handleRightClick();
    }
    players = players.filter((player) => player.entityType === EntityType.PLAYER);
    let percentages = players.map(player => (player.damageStats.damageDealt / encounterDamageStats.topDamageDealt) * 100);
    let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
    for (const [id, buff] of Object.entries(encounterDamageStats.buffs)) {
        if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
            continue;
        }
        if (buff.category === 'buff') {
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer);
        }
    }
    for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
        if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
            continue;
        }
        if (debuff.category === "debuff") {
            filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer);
        }
    }
    groupedSynergies = new Map([...groupedSynergies.entries()].sort());

    function filterStatusEffects(
        groupedSynergies: Map<string, Map<number, StatusEffect>>,
        buff: StatusEffect,
        id: number,
        focusedPlayer: Entity | null
    ) {
        // Party synergies
        if (['classskill', 'identity', 'ability'].includes(buff.buffCategory) &&
            buff.target === StatusEffectTarget.PARTY) {
            if (tab === MeterTab.PARTY_BUFFS) {
                const key = `${classesMap[buff.source.skill?.classId ?? 0]}_${buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name}`;
                groupedSynergiesAdd(groupedSynergies, key, id, buff);
            }     
        }
        // Self synergies
        else if (['pet', 'cook', 'battleitem', 'dropsofether', 'bracelet'].includes(buff.buffCategory)) {
            if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
                groupedSynergiesAdd(groupedSynergies, buff.buffCategory, id, buff);
            }
        } else if (['set'].includes(buff.buffCategory)) {
            if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
                groupedSynergiesAdd(groupedSynergies, `set_${buff.source.setName}`, id, buff);
            }
        } else if (['classskill', 'identity', 'ability'].includes(buff.buffCategory)) {
            // self & other identity, classskill, engravings
            if (tab === MeterTab.SELF_BUFFS && focusedPlayer) {
                let key;
                if (buff.buffCategory === 'ability') {
                    key = `${buff.uniqueGroup ? buff.uniqueGroup : id}`;
                } else {
                    if (focusedPlayer.classId !== buff.source.skill?.classId)
                        return; // We hide other classes self buffs (classskill & identity)
                    key = `${classesMap[buff.source.skill?.classId ?? 0]}_${buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name}`;
                }
                groupedSynergiesAdd(groupedSynergies, key, id, buff);
            }
        } else {
            // ignore
        }
    }
    
    function groupedSynergiesAdd(map: Map<string, Map<number, StatusEffect>>, key: string, id: number, buff: StatusEffect) {
        // by default, only show dmg, crit, atk spd, cd buffs.
        // show all arcana cards for fun
        if (!focusedPlayer || focusedPlayer.classId !== 202) {
            if (!defaultBuffFilter(buff.buffType)) {
                // console.log(buff);
                return;
            }
        }
        key = key.replaceAll(" ", "").toLowerCase();
        if (map.has(key)) {
            map.get(key)?.set(id, buff);
        } else {
            map.set(key, new Map([[id, buff]]));
        }
    }
</script>

<thead class="relative h-6 z-40" id="buff-head">
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal"></th>
        <th class="text-left px-2 font-normal w-20"></th>
        <th class="w-full"></th>
        {#each [...groupedSynergies] as [id, synergies] (id)}
            <LogBuffHeader {synergies} />
        {:else}
            <th class="font-normal w-20">No Buffs</th>
        {/each}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick} class="relative z-10">
    {#if !focusedPlayer}
    {#each players as player, i (player.name)}
        <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
            <LogBuffRow {player} {groupedSynergies} percentage={percentages[i]}/>
        </tr>
    {/each}
    {:else}
        <LogBuffBreakdown {groupedSynergies} player={focusedPlayer}/>
    {/if}
</tbody>
