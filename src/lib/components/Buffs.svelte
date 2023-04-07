<script lang="ts">
    import { classesMap } from '$lib/constants/classes';
    import {
        MeterTab,
        StatusEffectBuffTypeFlags,
        StatusEffectTarget,
        type EncounterDamageStats,
        type Entity,
        type StatusEffect
    } from '$lib/types';
    import { defaultBuffFilter } from '$lib/utils/buffs';
    import { join, resourceDir } from '@tauri-apps/api/path';
    import { convertFileSrc } from '@tauri-apps/api/tauri';
    import { flip } from 'svelte/animate';
    import BuffHeader from './BuffHeader.svelte';
    import BuffRow from './BuffRow.svelte';

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats | undefined;
    export let players: Array<Entity>;
    export let percentages: Array<number> = [];
    export let path: string;

    let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
    let focusedPlayer: Entity | null = null;

    $: {
        groupedSynergies = new Map<string, Map<number, StatusEffect>>();
        if (encounterDamageStats) {
            Object.entries(encounterDamageStats.buffs).forEach(([id, buff]) => {
                if (buff.source && buff.source.icon && !buff.source.icon.startsWith('http')) {
                    buff.source.icon = getIconPath(buff);
                }
                filterStatusEffects(buff, Number(id), focusedPlayer);
            });
            Object.entries(encounterDamageStats.debuffs).forEach(([id, debuff]) => {
                if (debuff.source && debuff.source.icon && !debuff.source.icon.startsWith('http')) {
                    debuff.source.icon = getIconPath(debuff);
                }
                filterStatusEffects(debuff, Number(id), focusedPlayer);
            });
            groupedSynergies = new Map([...groupedSynergies.entries()].sort());
            // console.log(groupedSynergies);
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

    function getIconPath(synergy: StatusEffect) {
        let fileName;
        if (synergy.source && synergy.source.icon) {
            fileName = synergy.source.icon;
        } else {
            fileName = "unknown.png";
        }
        return convertFileSrc(path + 'images\\skills\\' + fileName);
    }
</script>

<thead class="top-0 sticky h-6">
    <tr class="bg-zinc-900">
        <th class="w-7"></th>
        <th class="text-left px-2 font-normal w-full"></th>
            {#each [...groupedSynergies] as [id, synergies] (id)}
                <BuffHeader {synergies} />
            {:else}
                <th class="font-normal w-20">No Buffs</th>
            {/each}
    </tr>
</thead>
<tbody>
    {#each players as player, i (player.id)}
        <tr class="h-7 px-2 py-1" animate:flip={{ duration: 200 }}>
            <BuffRow {player} groupedSynergies={groupedSynergies} percentage={percentages[i]} />
        </tr>
    {/each}
</tbody>
