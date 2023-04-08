<script lang="ts">
    import { Buff, BuffDetails, type Skill, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { Tooltip } from 'flowbite-svelte';
    import BuffTooltipDetail from "./shared/BuffTooltipDetail.svelte";
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

    export let skill: Skill;
    export let color: string;
    export let damagePercentage: number;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;

    let synergyPercentageDetails: Array<BuffDetails>;

    const tweenedValue = tweened(0, {
    duration: 400,
    easing: cubicOut
    })


    $: {
        if (groupedSynergies.size > 0) {    
            tweenedValue.set(damagePercentage);   
            synergyPercentageDetails = [];
            groupedSynergies.forEach((synergies, key) => {
                let synergyDamage = 0;
                let buff = new BuffDetails();
                buff.id = key
                synergies.forEach((syn, id) => {
                    if (skill.buffedBy[id]) {
                        buff.buffs.push(new Buff(syn.source.icon, (skill.buffedBy[id] / skill.totalDamage * 100).toFixed(1), syn.source.skill?.icon));
                        synergyDamage += skill.buffedBy[id];
                    } else if (skill.debuffedBy[id]) {
                        buff.buffs.push(new Buff(syn.source.icon, (skill.debuffedBy[id] / skill.totalDamage * 100).toFixed(1), syn.source.skill?.icon));
                        synergyDamage += skill.debuffedBy[id];
                    }
                });
    
                if (synergyDamage > 0) {
                    buff.percentage = (synergyDamage / skill.totalDamage * 100).toFixed(1);
                }
                synergyPercentageDetails.push(buff);
            });        
        }   
    }

</script>

<td class="pl-1 relative z-10">
    <img class="h-5 w-5" src={skill.icon} alt={skill.name} />
</td>
<td class="relative z-10 -left-px">
    <div class="truncate">
        {skill.name}
    </div>
</td>
{#if groupedSynergies.size > 0}
{#each synergyPercentageDetails as synergy (synergy.id)}
    <td class="px-1 text-center">
        <div class="relative z-20">
            {synergy.percentage}<span class="text-3xs text-gray-300" class:hidden={!synergy.percentage}>%</span>
        </div>
        <Tooltip placement="bottom" defaultClass="bg-zinc-900 p-2 text-gray-300 z-50">
            <BuffTooltipDetail buffDetails={synergy} />
        </Tooltip>
    </td>
{/each}
{/if}
<div class="absolute left-0 h-7 px-2 py-1 z-0"
    style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
></div>
