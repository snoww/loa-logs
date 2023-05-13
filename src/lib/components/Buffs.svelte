<script lang="ts">
    import { classesMap } from '$lib/constants/classes';
    import {
        MeterTab,
        StatusEffectTarget,
        type EncounterDamageStats,
        type Entity,
        type StatusEffect,
        EntityType
    } from '$lib/types';
    import { defaultBuffFilter } from '$lib/utils/buffs';
    import { flip } from 'svelte/animate';
    import BuffHeader from './BuffHeader.svelte';
    import BuffRow from './BuffRow.svelte';
    import BuffSkillBreakdown from './BuffSkillBreakdown.svelte';

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats | undefined;
    export let players: Array<Entity>;
    export let percentages: Array<number> = [];
    export let focusedPlayer: Entity | null = null;
    export let handleRightClick: () => void;
    export let inspectPlayer: (name: string) => void;

    let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();

    $: {
        if (focusedPlayer && focusedPlayer.entityType === EntityType.ESTHER) {
            focusedPlayer = null;
            handleRightClick();
        }
        players = players.filter((player) => player.entityType === EntityType.PLAYER);
        groupedSynergies = new Map<string, Map<number, StatusEffect>>();
        if (encounterDamageStats) {
            Object.entries(encounterDamageStats.buffs).forEach(([id, buff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
                    return;
                }
                filterStatusEffects(buff, Number(id), focusedPlayer);
            });
            Object.entries(encounterDamageStats.debuffs).forEach(([id, debuff]) => {
                if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
                    return;
                }
                filterStatusEffects(debuff, Number(id), focusedPlayer);
            });
            groupedSynergies = new Map([...groupedSynergies.entries()].sort());
        }
    }

    function filterStatusEffects(
        buff: StatusEffect,
        id: number,
        focusedPlayer: Entity | null
    ) {
        // Party synergies
        if (['classskill', 'identity', 'ability'].includes(buff.buffCategory) &&
            buff.target === StatusEffectTarget.PARTY) {
            if (tab === MeterTab.PARTY_BUFFS) {
                const key = `${classesMap[buff.source.skill?.classId ?? 0]}_${buff.uniqueGroup ? buff.uniqueGroup : buff.source.skill?.name}`;
                groupedSynergiesAdd(key, id, buff);
            }     
        }
        // Self synergies
        else if (['pet', 'cook', 'battleitem', 'dropsofether', 'bracelet'].includes(buff.buffCategory)) {
            if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
                groupedSynergiesAdd(buff.buffCategory, id, buff);
            }
        } else if (['set'].includes(buff.buffCategory)) {
            if (tab === MeterTab.SELF_BUFFS && !focusedPlayer) {
                groupedSynergiesAdd(`set_${buff.source.setName}`, id, buff);
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
                groupedSynergiesAdd(key, id, buff);
            }
        } else {
            // ignore
        }
    }
    
    function groupedSynergiesAdd(key: string, id: number, buff: StatusEffect) {
        // by default, only show dmg, crit, atk spd, cd buffs.
        if (!defaultBuffFilter(buff.buffType)) {
            // console.log(buff);
            return;
        }
        key = key.replace(" ", "").toLowerCase();
        if (groupedSynergies.has(key)) {
            groupedSynergies.get(key)?.set(id, buff);
        } else {
            groupedSynergies.set(key, new Map([[id, buff]]));
        }
    }
</script>

<thead class="top-0 sticky h-6 z-40">
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal"></th>
        <th class="text-left px-2 font-normal w-14"></th>
        <th class="w-full"></th>
        {#each [...groupedSynergies] as [id, synergies] (id)}
            <BuffHeader {synergies} />
        {:else}
            <th class="font-normal w-20">No Buffs</th>
        {/each}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#if !focusedPlayer}
    {#each players as player, i (player.id)}
        <tr class="h-7 px-2 py-1" animate:flip={{ duration: 200 }} on:click={() => inspectPlayer(player.name)}>
            <BuffRow {player} {groupedSynergies} percentage={percentages[i]} />
        </tr>
    {/each}
    {:else}
        <BuffSkillBreakdown {groupedSynergies} player={focusedPlayer}/>
    {/if}
</tbody>
