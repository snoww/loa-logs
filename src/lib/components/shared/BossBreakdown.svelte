<script lang="ts">
    import type { Entity, Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";
    import BossBreakdownRow from "./BossBreakdownRow.svelte";

    interface Props {
        boss: Entity | undefined;
        duration: number;
        handleRightClick: () => void;
        tween?: boolean;
    }

    let { boss, duration, handleRightClick, tween = true }: Props = $props();

    let skills: Array<Skill> = $state([]);
    let skillDamagePercentages: Array<number> = $state([]);
    let abbreviatedSkillDamage: Array<(string | number)[]> = $derived(
        skills.map((skill) => abbreviateNumberSplit(skill.totalDamage))
    );
    let skillDps: Array<(string | number)[]> = $derived(
        skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)))
    );

    $effect(() => {
        if (boss) {
            skills = Object.values(boss.skills).sort((a, b) => b.totalDamage - a.totalDamage);
        }
    });
    $effect(() => {
        if (skills.length > 0) {
            let mostDamageSkill = skills[0].totalDamage;
            skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
        }
    });
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tighter">
            <th class="w-14 px-2 text-left font-normal"></th>
            <th class="w-full"></th>
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
        </tr>
    </thead>
    <tbody oncontextmenu={handleRightClick} class="relative z-10">
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
                        {tween} />
                </tr>
            {/each}
        {/if}
    </tbody>
</table>
