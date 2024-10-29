<script lang="ts">
    import type { Skill } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";
    import PlayerBreakdownRow from "./shared/PlayerBreakdownRow.svelte";

    let {
        skill,
        color,
        hasFrontAttacks,
        hasBackAttacks,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        abbreviatedSkillDamage,
        skillDps,
        playerDamageDealt,
        damagePercentage,
        duration,
        index
    }: {
        skill: Skill;
        color: string;
        hasFrontAttacks: boolean;
        hasBackAttacks: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        abbreviatedSkillDamage: (string | number)[];
        skillDps: (string | number)[];
        playerDamageDealt: number;
        damagePercentage: number;
        duration: number;
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
    {index} />
