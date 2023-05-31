<script lang="ts">
    import { takingScreenshot } from "$lib/utils/stores";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "../shared/BuffSkillBreakdownRow.svelte";

    export let skill: Skill;
    export let color: string;
    export let damagePercentage: number;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
</script>

<BuffSkillBreakdownRow {skill} {color} {groupedSynergies} width={damagePercentage} shadow={!$takingScreenshot} />
