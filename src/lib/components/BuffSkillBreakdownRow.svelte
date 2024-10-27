<script lang="ts">
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "./shared/BuffSkillBreakdownRow.svelte";

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

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(damagePercentage);
    });
</script>

<BuffSkillBreakdownRow {skill} {color} {groupedSynergies} width={$tweenedValue} {index} />
