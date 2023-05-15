<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "./BuffSkillBreakdownRow.svelte";

    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let player: Entity;

    let color: string;
    let skillDamagePercentages: Array<number> = [];
    let skills = Array<Skill>();

    $: {
        skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);
        if (Object.hasOwn(classColors, player.class)) {
            color = classColors[player.class].color;
        }

        if (skills.length > 0) {
            let mostDamageSkill = skills[0].totalDamage;
            skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
        }
    }
</script>

{#each skills as skill, i (skill.id)}
    <BuffSkillBreakdownRow {groupedSynergies} {skill} {color} damagePercentage={skillDamagePercentages[i]} />
{/each}
