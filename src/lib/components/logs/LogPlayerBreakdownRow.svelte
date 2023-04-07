<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";

    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;

    let critPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    if (skill.hits !== 0) {
        critPercentage = (skill.crits / skill.hits * 100).toFixed(1);
        faPercentage = (skill.frontAttacks / skill.hits * 100).toFixed(1) ;
        baPercentage = (skill.backAttacks / skill.hits * 100).toFixed(1);
    }

</script>


<td class="px-1 relative z-10">
    <div class="flex space-x-1 items-center">
        <img class="h-5 w-5" src={skill.icon} alt={skill.name} />
        <div class="truncate">
            {skill.name}
        </div>
    </div>
</td>
<td class="px-1 text-center relative z-10">
    {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
</td>
<td class="px-1 text-center relative z-10">
    {(skill.totalDamage / playerDamageDealt * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
</td>
<td class="px-1 text-center relative z-10">
    {critPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{#if hasFrontAttacks}
<td class="px-1 text-center relative z-10">
    {faPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
{#if hasBackAttacks}
<td class="px-1 text-center relative z-10">
    {baPercentage}<span class="text-3xs text-gray-300">%</span>
</td>
{/if}
<td class="px-1 text-center relative z-10">
    {skill.casts.toLocaleString()}
</td>
<div class="absolute left-0 h-7 px-2 py-1 z-0"
    style="background-color: {HexToRgba(color, 0.6)}; width: {damagePercentage}%"
></div>
