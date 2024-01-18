<script lang="ts">
    import type { Skill } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import PlayerBreakdownRow from "./shared/PlayerBreakdownRow.svelte";

    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let anySupportBuff: boolean;
    export let anySupportIdentity: boolean;
    export let anySupportBrand: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;
    export let duration: number;
    export let index: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    });

    $: {
        tweenedValue.set(damagePercentage);
    }
</script>

<PlayerBreakdownRow
    {skill}
    {color}
    {hasFrontAttacks}
    {hasBackAttacks}
    {anySupportBuff}
    {anySupportIdentity}
    {anySupportBrand}
    {abbreviatedSkillDamage}
    {skillDps}
    {playerDamageDealt}
    {duration}
    width={$tweenedValue}
    meterSettings={$settings.meter}
    {index}
    />
