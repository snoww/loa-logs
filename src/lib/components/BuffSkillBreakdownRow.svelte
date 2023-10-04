<script lang="ts">
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import type { Skill, StatusEffect } from "$lib/types";
    import BuffSkillBreakdownRow from "./shared/BuffSkillBreakdownRow.svelte";

    export let skill: Skill;
    export let color: string;
    export let damagePercentage: number;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let index: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $: {
        tweenedValue.set(damagePercentage);
    }
</script>

<BuffSkillBreakdownRow {skill} {color} {groupedSynergies} width={$tweenedValue} {index}/>
