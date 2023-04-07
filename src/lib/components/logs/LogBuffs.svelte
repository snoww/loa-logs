<script lang="ts">
    import { classesMap } from "$lib/constants/classes";
    import { StatusEffectTarget, type EncounterDamageStats, type Entity, MeterTab, type StatusEffect } from "$lib/types";
    import { defaultBuffFilter } from "$lib/utils/buffs";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import LogBuffHeader from "./LogBuffHeader.svelte";
    import LogBuffRow from "./LogBuffRow.svelte";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats;
    export let players: Array<Entity>;
    export let percentages: Array<number> = [];
    export let classIconsCache: { [key: number]: string };
    let focusedPlayer: Entity | null = null;


    async function processBuffs() {        
        let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
        for (const [id, buff] of Object.entries(encounterDamageStats.buffs)) {
            if (buff.source && buff.source.icon && !buff.source.icon.startsWith('http')) {
                buff.source.icon = await getIconPath(buff);
            }
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer);
        }
        for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
            if (debuff.source && debuff.source.icon && !debuff.source.icon.startsWith('http')) {
                debuff.source.icon = await getIconPath(debuff);
            }
            filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer);
        }
        groupedSynergies = new Map([...groupedSynergies.entries()].sort());
        return groupedSynergies
    }

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
        if (!defaultBuffFilter(buff.buffType)) {
            // console.log(buff);
            return;
        }
        key = key.replaceAll(" ", "").toLowerCase();
        if (map.has(key)) {
            map.get(key)?.set(id, buff);
        } else {
            map.set(key, new Map([[id, buff]]));
        }
    }

    async function getIconPath(synergy: StatusEffect) {
        let fileName;
        if (synergy.source && synergy.source.icon) {
            fileName = synergy.source.icon;
        } else {
            fileName = "unknown.png";
        }
        return convertFileSrc(await join(await resourceDir(), 'images', 'skills', fileName));
    }
</script>

{#await processBuffs() then groupedSynergies}
<thead class="top-0 sticky h-6" id="buff-head">
    <tr class="bg-zinc-900">
        <th class="w-7"></th>
        <th class="text-left px-2 font-normal w-full"></th>
            {#each [...groupedSynergies] as [id, synergies] (id)}
                <LogBuffHeader {synergies} />
            {:else}
                <th class="font-normal w-20">No Buffs</th>
            {/each}
    </tr>
</thead>
<tbody>
    {#each players as player, i (player.name)}
        <tr class="h-7 px-2 py-1">
            <LogBuffRow {player} groupedSynergies={groupedSynergies} percentage={percentages[i]} classIconsCache={classIconsCache}/>
        </tr>
    {/each}
</tbody>
{/await}
