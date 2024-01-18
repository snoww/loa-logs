<script lang="ts">
    import type { Skill } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import PlayerBreakdownRow from "../shared/PlayerBreakdownRow.svelte";

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
        width={damagePercentage}
        meterSettings={$settings.logs}
        shadow={!$takingScreenshot} 
        {index}/>
</tr>
