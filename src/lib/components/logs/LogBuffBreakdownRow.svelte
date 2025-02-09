<script lang="ts">
    import { takingScreenshot } from "$lib/utils/stores";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "../shared/BuffSkillBreakdownRow.svelte";

    interface Props {
        skill: Skill;
        color: string;
        damagePercentage: number;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        index: number;
    }

    let { skill, color, damagePercentage, groupedSynergies, index }: Props = $props();
</script>

<BuffSkillBreakdownRow
    {skill}
    {color}
    {groupedSynergies}
    width={damagePercentage}
    shadow={!$takingScreenshot}
    {index} />
