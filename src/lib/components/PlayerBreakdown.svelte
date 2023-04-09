<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { cubicOut } from "svelte/easing";
    import { tweened, type Tweened } from "svelte/motion";
    import { flip } from 'svelte/animate';
    import PlayerBreakdownRow from "./PlayerBreakdownRow.svelte";


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
        <th class="text-left px-2 font-normal w-full">Skill</th>
        <th class="font-normal w-14">DMG</th>
        <th class="font-normal w-14">DPS</th>
        <th class="font-normal w-14">D%</th>
        <th class="font-normal w-14">Crit</th>
        {#if hasFrontAttacks}
        <th class="font-normal w-14">F.A</th>
        {/if}
        {#if hasBackAttacks}
        <th class="font-normal w-14">B.A</th>
        {/if}
        <th class="font-normal w-14">Casts</th>
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
            />
    </tr>
    {/each}
    {/if}
</tbody>

