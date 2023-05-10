<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import LogPlayerBreakdownRow from "./LogPlayerBreakdownRow.svelte";
    import { settings } from "$lib/utils/settings";

    export let entity: Entity;
    export let duration: number;
    export let handleRightClick: () => void;

    let color = "#ffffff";
    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];
    
    let hasBackAttacks = false;
    let hasFrontAttacks = false;
    let anySupportBrand = false;
    let anySupportBuff = false;

    skills = Object.values(entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
    
    if (Object.hasOwn(classColors, entity.class)){
        color = classColors[entity.class].color;
    } else if (entity.entityType === EntityType.ESTHER) {
        color = "#4dc8d0";
    }

    if (skills.length > 0) {
        let mostDamageSkill = skills[0].totalDamage;
        skillDamagePercentages = skills.map(skill => (skill.totalDamage / mostDamageSkill) * 100);
        abbreviatedSkillDamage = skills.map(skill => abbreviateNumberSplit(skill.totalDamage));
        skillDps = skills.map(skill => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
        hasBackAttacks = skills.some(skill => skill.backAttacks > 0);
        hasFrontAttacks = skills.some(skill => skill.frontAttacks > 0);
        anySupportBuff = skills.some(skill => skill.buffedBySupport > 0);
        anySupportBrand = skills.some(skill => skill.debuffedBySupport > 0);
    }
</script>

<thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
    <tr class="bg-zinc-900">
        <th class="text-left px-2 font-normal w-full"></th>
        {#if $settings.logs.breakdown.damage}
        <th class="font-normal w-14">DMG</th>
        {/if}
        {#if $settings.logs.breakdown.dps}
        <th class="font-normal w-14">DPS</th>
        {/if}
        {#if $settings.logs.breakdown.damagePercent}
        <th class="font-normal w-14">D%</th>
        {/if}
        {#if $settings.logs.breakdown.critRate}
        <th class="font-normal w-14">CRIT</th>
        {/if}
        {#if hasFrontAttacks && $settings.logs.breakdown.frontAtk}
        <th class="font-normal w-14">F.A</th>
        {/if}
        {#if hasBackAttacks && $settings.logs.breakdown.backAtk}
        <th class="font-normal w-14">B.A</th>
        {/if}
        {#if anySupportBuff && $settings.logs.breakdown.percentBuffBySup}
        <th class="font-normal w-14">Buff%</th>
        {/if}
        {#if anySupportBrand && $settings.logs.breakdown.percentBrand}
        <th class="font-normal w-16">Brand%</th>
        {/if}
        {#if $settings.logs.breakdown.avgDamage}
        <th class="font-normal w-14">Avg</th>
        {/if}
        {#if $settings.logs.breakdown.maxDamage}
        <th class="font-normal w-14">Max</th>
        {/if}
        {#if $settings.logs.breakdown.casts}
        <th class="font-normal w-16">Casts/m</th>
        {/if}
        {#if $settings.logs.breakdown.hits}
        <th class="font-normal w-14">Hits/m</th>
        {/if}
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#each skills as skill, i (skill.id)}
    <tr class="h-7 px-2 py-1 text-3xs">
        <LogPlayerBreakdownRow
            {skill}
            {color}
            {hasFrontAttacks}
            {hasBackAttacks} 
            {anySupportBuff}
            {anySupportBrand}
            abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
            playerDamageDealt={entity.damageStats.damageDealt}
            damagePercentage={skillDamagePercentages[i]}
            skillDps={skillDps[i]}
            duration={duration}
            />
    </tr>
    {/each}
</tbody>

