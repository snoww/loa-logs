<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import LogPlayerBreakdownRow from "./LogPlayerBreakdownRow.svelte";
    import { colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "../shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";

    export let entity: Entity;
    export let duration: number;

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
    if (entity.class === "Arcanist") {
        skills = skills.filter((skill) => !cardIds.includes(skill.id));
    }

    if (Object.hasOwn($colors, entity.class)) {
        color = $colors[entity.class].color;
    } else if (entity.entityType === EntityType.ESTHER) {
        color = "#4dc8d0";
    }

    if (skills.length > 0) {
        let mostDamageSkill = skills[0].totalDamage;
        skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
        abbreviatedSkillDamage = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage));
        skillDps = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
        hasBackAttacks = skills.some((skill) => skill.backAttacks > 0);
        hasFrontAttacks = skills.some((skill) => skill.frontAttacks > 0);
        anySupportBuff = skills.some((skill) => skill.buffedBySupport > 0);
        anySupportBrand = skills.some((skill) => skill.debuffedBySupport > 0);
    }
</script>

<thead
    class="z-30 h-6"
    on:contextmenu|preventDefault={() => {
        console.log("titlebar clicked");
    }}>
    <tr class="bg-zinc-900">
        <PlayerBreakdownHeader
            meterSettings={$settings.logs}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
            {anySupportBrand} />
    </tr>
</thead>
<tbody class="relative z-10">
    {#each skills as skill, i (skill.id)}
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
            {duration} />
    {/each}
</tbody>
