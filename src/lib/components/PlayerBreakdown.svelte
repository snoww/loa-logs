<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { flip } from 'svelte/animate';
    import PlayerBreakdownRow from "./PlayerBreakdownRow.svelte";
    import { settings } from "$lib/utils/settings";

    export let player: Entity | null;
    export let duration: number;
    export let handleRightClick: () => void;

    let color = "#ffffff";
    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];
    
    let hasBackAttacks = true;
    let hasFrontAttacks = true;

    $: {
        if (player) {
            if (Object.hasOwn(classColors, player.class)){
                color = classColors[player.class].color;
            }
            skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);
            if (skills.length > 0) {
                let mostDamageSkill = skills[0].totalDamage;
                skillDamagePercentages = skills.map(skill => (skill.totalDamage / mostDamageSkill) * 100);
                abbreviatedSkillDamage = skills.map(skill => abbreviateNumberSplit(skill.totalDamage));
                skillDps = skills.map(skill => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
                hasBackAttacks = skills.some(skill => skill.backAttacks > 0);
                hasFrontAttacks = skills.some(skill => skill.frontAttacks > 0);
            }            
        }
    }
</script>

<thead class="top-0 sticky h-6">
    <tr class="bg-zinc-900">
        <th class="w-7"></th>
        <th class="text-left px-2 font-normal w-full"></th>
        {#if $settings.meter.breakdown.damage}
        <th class="font-normal w-14">DMG</th>
        {/if}
        {#if $settings.meter.breakdown.dps}
        <th class="font-normal w-14">DPS</th>
        {/if}
        {#if $settings.meter.breakdown.damagePercent}
        <th class="font-normal w-14">D%</th>
        {/if}
        {#if $settings.meter.breakdown.critRate}
        <th class="font-normal w-14">Crit</th>
        {/if}
        {#if hasFrontAttacks && $settings.meter.breakdown.frontAtk}
        <th class="font-normal w-14">F.A</th>
        {/if}
        {#if hasBackAttacks && $settings.meter.breakdown.backAtk}
        <th class="font-normal w-14">B.A</th>
        {/if}
        {#if $settings.meter.breakdown.avgDamage}
        <th class="font-normal w-14">Avg</th>
        {/if}
        {#if $settings.meter.breakdown.maxDamage}
        <th class="font-normal w-14">Max</th>
        {/if}
        {#if $settings.meter.breakdown.casts}
        <th class="font-normal w-16">Casts/m</th>
        {/if}
        {#if $settings.meter.breakdown.hits}
        <th class="font-normal w-14">Hits/m</th>
        {/if}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#if player}
    {#each skills as skill, i (skill.id)}
    <tr class="h-7 px-2 py-1 text-3xs" animate:flip="{{duration: 200}}">
        <PlayerBreakdownRow
            {skill}
            {color}
            {hasFrontAttacks}
            {hasBackAttacks} 
            abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
            playerDamageDealt={player.damageStats.damageDealt}
            damagePercentage={skillDamagePercentages[i]}
            skillDps={skillDps[i]}
            {duration}
            />
    </tr>
    {/each}
    {/if}
</tbody>

