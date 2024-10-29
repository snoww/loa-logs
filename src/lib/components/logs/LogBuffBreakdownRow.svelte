<script lang="ts">
    import { takingScreenshot } from "$lib/utils/stores";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "../shared/BuffSkillBreakdownRow.svelte";

    let {
        skill,
        color,
        damagePercentage,
        groupedSynergies,
        index
    }: {
        skill: Skill;
        color: string;
        damagePercentage: number;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        index: number;
    } = $props();
</script>

<BuffSkillBreakdownRow
    {skill}
    {color}
    {groupedSynergies}
    width={damagePercentage}
    shadow={!$takingScreenshot}
    {index} />
