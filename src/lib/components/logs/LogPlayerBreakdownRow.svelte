<script lang="ts">
    import type { Skill } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import PlayerBreakdownRow from "../shared/PlayerBreakdownRow.svelte";

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
        playerDamageDealt: number;
        damagePercentage: number;
        duration: number;
        index: number;
        skillDpsRaw: number;
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
        playerDamageDealt,
        damagePercentage,
        duration,
        index,
        skillDpsRaw
    }: Props = $props();
</script>

<tr class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}">
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
        {skillDpsRaw}
        width={damagePercentage}
        meterSettings={"logs"}
        shadow={!$takingScreenshot}
        {index} />
</tr>
