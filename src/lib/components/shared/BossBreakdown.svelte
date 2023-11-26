<script lang="ts">
    import type { Entity, Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";
    import BossBreakdownRow from "./BossBreakdownRow.svelte";

    export let boss: Entity | undefined;
    export let duration: number;
    export let handleRightClick: () => void;
    export let tween = true;

    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];

    $: {
        if (boss) {
            skills = Object.values(boss.skills).sort((a, b) => b.totalDamage - a.totalDamage);
            if (skills.length > 0) {
                let mostDamageSkill = skills[0].totalDamage;
                skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
                abbreviatedSkillDamage = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage));
                skillDps = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
            }
        }
    }
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tighter">
            <th class="w-14 px-2 text-left font-normal" />
            <th class="w-full" />
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
        </tr>
    </thead>
    <tbody on:contextmenu|preventDefault={handleRightClick} class="relative z-10">
        {#if boss}
            {#each skills as skill, i (skill.id)}
                <tr
                    class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                    animate:flip={{ duration: 200 }}>
                    <BossBreakdownRow
                        {skill}
                        abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
                        skillDps={skillDps[i]} 
                        width={skillDamagePercentages[i]}
                        index={i}
                        {duration}
                        totalDamageDealt={boss.damageStats.damageDealt}
                        {tween}
                        />
                </tr>
            {/each}
        {/if}
    </tbody>
</table>
