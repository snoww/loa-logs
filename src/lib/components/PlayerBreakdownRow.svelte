<script lang="ts">
    import type { Skill } from "$lib/types";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import PlayerBreakdownRow from "./shared/PlayerBreakdownRow.svelte";

    interface Props {
        skill: Skill;
        color: string;
        hasFrontAttacks: boolean;
        hasBackAttacks: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        abbreviatedSkillDamage: (string | number)[];
        skillDps: (string | number)[];
        skillDpsRaw: number;
        playerDamageDealt: number;
        damagePercentage: number;
        duration: number;
        index: number;
    }

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
        skillDpsRaw,
        playerDamageDealt,
        damagePercentage,
        duration,
        index
    }: Props = $props();

    const tweenedValue = new Tween(0, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(damagePercentage ?? 0);
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
    {skillDpsRaw}
    {playerDamageDealt}
    {duration}
    width={tweenedValue.current}
    meterSettings={"meter"}
    {index} />
