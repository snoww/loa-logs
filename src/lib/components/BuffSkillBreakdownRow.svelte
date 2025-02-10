<script lang="ts">
    import { cubicOut } from "svelte/easing";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "./shared/BuffSkillBreakdownRow.svelte";
    import { Tween } from "svelte/motion";

    interface Props {
        skill: Skill;
        color: string;
        damagePercentage: number;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        index: number;
    }

    let { skill, color, damagePercentage, groupedSynergies, index }: Props = $props();

    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });
    $effect(() => {
        tweenedValue.set(damagePercentage ?? 0);
    });
</script>

<BuffSkillBreakdownRow {skill} {color} {groupedSynergies} width={tweenedValue.current} {index} />
