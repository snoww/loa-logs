<script lang="ts">
    import { classesMap } from "$lib/constants/classes";
    import { StatusEffectTarget, type EncounterDamageStats, type Entity, MeterTab, type StatusEffect } from "$lib/types";
    import { defaultBuffFilter } from "$lib/utils/buffs";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import LogBuffHeader from "./LogBuffHeader.svelte";
    import LogBuffRow from "./LogBuffRow.svelte";
    import LogBuffBreakdown from "./LogBuffBreakdown.svelte";

    export let tab: MeterTab;
    export let encounterDamageStats: EncounterDamageStats;
    export let players: Array<Entity>;
    export let percentages: Array<number> = [];
    export let classIconsCache: { [key: number]: string };
    export let focusedPlayer: Entity | null = null;
    export let handleRightClick: () => void;
    export let inspectPlayer: (name: string) => void;

    let resourcePath: string | null = null;


    async function processBuffs() {        
        let groupedSynergies: Map<string, Map<number, StatusEffect>> = new Map();
        for (const [id, buff] of Object.entries(encounterDamageStats.buffs)) {
            if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.buffedBy, id)) {
                continue;
            }
            if (buff.category === 'buff') {
            if (buff.source && buff.source.icon && !buff.source.icon.startsWith('http')) {
                buff.source.icon = await getPath(buff.source.icon);
            } else if (buff.source && !buff.source.icon) {
                buff.source.icon = await getPath("unknown.png");
            }
            if (buff.source.skill && buff.source.skill.icon && !buff.source.skill.icon.startsWith('http')) {
                buff.source.skill.icon = await getPath(buff.source.skill.icon);
            } else if (buff.source.skill && !buff.source.skill.icon) {
                buff.source.skill.icon = await getPath("unknown.png");
            }
            
            filterStatusEffects(groupedSynergies, buff, Number(id), focusedPlayer);
        }
        }
        for (const [id, debuff] of Object.entries(encounterDamageStats.debuffs)) {
            if (focusedPlayer && !Object.hasOwn(focusedPlayer.damageStats.debuffedBy, id)) {
                continue;
            }
            if (debuff.category === "debuff") {
                if (debuff.source && debuff.source.icon && !debuff.source.icon.startsWith('http')) {
                    debuff.source.icon = await getPath(debuff.source.icon);
                } else if (debuff.source && !debuff.source.icon) {
                    debuff.source.icon = await getPath("unknown.png");
                }
                if (debuff.source.skill && debuff.source.skill.icon && !debuff.source.skill.icon.startsWith('http')) {
                    debuff.source.skill.icon = await getPath(debuff.source.skill.icon);
                } else if (debuff.source.skill && !debuff.source.skill.icon) {
                    debuff.source.skill.icon = await getPath("unknown.png");
                }
                filterStatusEffects(groupedSynergies, debuff, Number(id), focusedPlayer);
            }
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

    async function getPath(icon: string) {
        return convertFileSrc(await join(await resourceDir(), 'images', 'skills', icon));
    }
</script>

{#await processBuffs() then groupedSynergies}
<thead class="relative h-6 z-50" id="buff-head">
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal"></th>
        <th class="text-left px-2 font-normal w-full"></th>
        {#each [...groupedSynergies] as [id, synergies] (id)}
            <LogBuffHeader {synergies} />
        {:else}
            <th class="font-normal w-20">No Buffs</th>
        {/each}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#if !focusedPlayer}
    {#each players as player, i (player.name)}
        <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
            <LogBuffRow {player} groupedSynergies={groupedSynergies} percentage={percentages[i]} classIconsCache={classIconsCache}/>
        </tr>
    {/each}
    {:else}
        <LogBuffBreakdown groupedSynergies={groupedSynergies} player={focusedPlayer}/>
    {/if}
</tbody>

{/await}
